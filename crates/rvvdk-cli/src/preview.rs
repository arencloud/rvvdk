use crate::error::Failure;
use crate::source::Source as OpenedSource;
use rvvdk_datamover::{CopyOptions, CopyPlan, DataMover};
use serde::Serialize;
use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Failure>;

#[derive(Serialize)]
pub(crate) struct PathReport {
    display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes_hex: Option<String>,
}
impl PathReport {
    pub fn label(&self) -> String {
        format!("{:?}", self.display)
    }
}
impl From<&Path> for PathReport {
    fn from(path: &Path) -> Self {
        Self {
            display: path.to_string_lossy().into_owned(),
            bytes_hex: path.to_str().is_none().then(|| {
                path.as_os_str()
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect()
            }),
        }
    }
}
#[derive(Serialize)]
pub(crate) struct Identity {
    device: u64,
    inode: u64,
}
#[derive(Serialize)]
pub(crate) struct Source {
    pub path: PathReport,
    pub logical_bytes: u64,
    pub logical_block_size: u32,
    pub physical_block_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity: Option<Identity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vmdk: Option<serde_json::Value>,
}
#[derive(Serialize)]
pub(crate) struct Destination {
    pub path: PathReport,
    pub state: &'static str,
    pub policy: &'static str,
    pub existing_bytes: Option<u64>,
    pub copy_range_bytes: u64,
    pub preserved_tail_bytes: u64,
    pub write_access_checked: bool,
}
#[derive(Serialize)]
pub(crate) struct Execution {
    pub requested_backend: String,
    pub selected_backend: Option<&'static str>,
    pub selection_reason: &'static str,
    pub runtime_prepared: bool,
    pub block_size: usize,
    pub buffer_alignment: usize,
    pub workers: usize,
    pub work_queue_capacity: usize,
    pub native_queue_depth: u32,
    pub native_read_window: usize,
    pub memory_budget_bytes: usize,
    pub threaded_payload_estimate_bytes: usize,
    pub threaded_payload_fits_budget: bool,
}
pub(crate) struct Preview {
    pub command: &'static str,
    pub format: &'static str,
    pub source: Source,
    pub plan: CopyPlan,
    pub destination: Option<Destination>,
    pub execution: Option<Execution>,
    pub include_extents: bool,
}
pub(crate) fn build(name: &str, args: &clap::ArgMatches) -> Result<Preview> {
    let source_path = args.get_one::<PathBuf>("source").expect("required source");
    let opened = OpenedSource::open(source_path, args.get_one::<String>("format").unwrap())?;
    let source = opened.logical();
    let endpoint = source.copy_endpoint()?;
    let (device, inode) = opened.identity();
    if name == "plan" {
        opened.validate_backend(args.get_one::<String>("backend").unwrap())?;
    }
    let budget = *args.get_one::<usize>("memory-budget").unwrap();
    let options = if name == "plan" {
        CopyOptions::with_concurrency(
            *args.get_one("block-size").unwrap(),
            4096,
            *args.get_one("workers").unwrap(),
        )?
    } else {
        CopyOptions::default()
    }
    .with_memory_budget(budget);
    // Reuse canonical logical topology validation and metadata budget admission.
    // This portable selection is NOT presented as the future RAW native decision.
    let mover = DataMover::new(options);
    let plan = mover.plan(source)?;
    let live = source.copy_endpoint()?;
    if live.size != source.size() || live.identity != endpoint.identity {
        return Err(Failure::new(
            "source_changed",
            "source size or identity changed during inspection",
        ));
    }
    opened.revalidate()?;
    let mut preview = Preview {
        format: opened.format(),
        command: if name == "plan" { "plan" } else { "inspect" },
        source: Source {
            path: source_path.as_path().into(),
            logical_bytes: source.size(),
            logical_block_size: source.geometry().logical_block_size(),
            physical_block_size: source.geometry().physical_block_size(),
            identity: (opened.format() == "raw").then_some(Identity { device, inode }),
            vmdk: opened.vmdk_report(),
        },
        plan,
        destination: None,
        execution: None,
        include_extents: args.get_flag("extents"),
    };
    if name == "plan" {
        let destination = args.get_one::<PathBuf>("destination").unwrap();
        preview.destination = Some(destination_preview(
            destination,
            args.get_flag("overwrite"),
            &opened,
            source.size(),
        )?);
        let requested = args.get_one::<String>("backend").unwrap();
        let estimate = mover.execution_memory(&preview.plan)?.total_bytes();
        let queue = *args.get_one::<u32>("queue-depth").unwrap();
        preview.execution = Some(Execution {
            requested_backend: requested.clone(),
            selected_backend: (requested == "threaded" || opened.format() == "vmdk")
                .then_some("threaded"),
            selection_reason: if requested == "threaded" {
                "requested_threaded"
            } else if opened.format() == "vmdk" {
                "portable_api"
            } else {
                "deferred_until_destination_preparation"
            },
            runtime_prepared: false,
            block_size: options.block_size(),
            buffer_alignment: options.buffer_alignment(),
            workers: options.concurrency(),
            work_queue_capacity: options.queue_capacity(),
            native_queue_depth: queue,
            native_read_window: (queue as usize).div_ceil(2),
            memory_budget_bytes: budget,
            threaded_payload_estimate_bytes: estimate,
            threaded_payload_fits_budget: estimate <= budget,
        });
    }
    Ok(preview)
}
fn destination_preview(
    path: &Path,
    overwrite: bool,
    source: &OpenedSource,
    size: u64,
) -> Result<Destination> {
    let bytes = path.as_os_str().as_bytes();
    if path.file_name().is_none() || bytes.ends_with(b"/") || bytes.ends_with(b"/.") {
        return Err(Failure::new(
            "invalid_destination",
            "destination must name a file",
        ));
    }
    let existing = match fs::symlink_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Failure::io(&format!("inspect destination {path:?}"), e)),
    };
    if let Some(metadata) = &existing {
        // Reject leaf symlinks, including dangling links, even with --overwrite.
        if !metadata.is_file() {
            return Err(Failure::new(
                "invalid_destination",
                "destination must be a regular file, not a symlink or special file",
            ));
        }
        source.validate_destination(metadata)?;
        if !overwrite {
            return Err(Failure::new(
                "destination_exists",
                "destination exists; use --overwrite to preview an in-place overwrite",
            ));
        }
        if metadata.len() < size {
            return Err(Failure::new(
                "destination_too_small",
                "in-place destination is smaller than the source; preview does not resize it",
            ));
        }
    } else {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if path.file_name().is_none() {
            return Err(Failure::new(
                "invalid_destination",
                "destination needs a file name",
            ));
        }
        let metadata = fs::metadata(parent)
            .map_err(|e| Failure::io(&format!("inspect destination parent {parent:?}"), e))?;
        if !metadata.is_dir() {
            return Err(Failure::new(
                "invalid_destination",
                "destination parent is not a directory",
            ));
        }
    }
    Ok(Destination {
        path: path.into(),
        state: if existing.is_some() {
            "existing"
        } else {
            "new"
        },
        policy: if existing.is_some() {
            "overwrite_in_place"
        } else {
            "create_new_no_clobber"
        },
        existing_bytes: existing.as_ref().map(fs::Metadata::len),
        copy_range_bytes: size,
        preserved_tail_bytes: existing.as_ref().map_or(0, |m| m.len() - size),
        write_access_checked: false,
    })
}
