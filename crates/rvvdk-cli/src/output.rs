use crate::{error::Failure, preview::Preview};
use rvvdk_core::{Extent, ExtentKind};
use serde::{Serialize, Serializer, ser::SerializeSeq};
use std::io::Write;

#[derive(Serialize)]
struct Summary {
    logical_bytes: u64,
    data_bytes: u64,
    zero_bytes: u64,
    hole_bytes: u64,
    extent_count: usize,
}
#[derive(Serialize)]
struct ExtentReport {
    offset: u64,
    length: u64,
    kind: &'static str,
}
fn extent(e: &Extent) -> ExtentReport {
    ExtentReport {
        offset: e.offset(),
        length: e.length(),
        kind: match e.kind() {
            ExtentKind::Data => "data",
            ExtentKind::Zero => "zero",
            ExtentKind::Hole => "hole",
        },
    }
}
struct Extents<'a>(&'a [Extent]);
impl Serialize for Extents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            seq.serialize_element(&extent(item))?;
        }
        seq.end()
    }
}
#[derive(Serialize)]
struct Report<'a> {
    schema_version: u32,
    command: &'static str,
    format: &'static str,
    status: &'static str,
    source: &'a crate::preview::Source,
    summary: Summary,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination: &'a Option<crate::preview::Destination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution: &'a Option<crate::preview::Execution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extents: Option<Extents<'a>>,
}
pub(crate) fn write(preview: &Preview, json: bool, out: &mut impl Write) -> Result<(), Failure> {
    let plan = &preview.plan;
    if json {
        let report = Report {
            schema_version: 1,
            command: preview.command,
            format: "raw",
            status: if preview.command == "plan" {
                "preview"
            } else {
                "inspected"
            },
            source: &preview.source,
            summary: Summary {
                logical_bytes: plan.logical_bytes(),
                data_bytes: plan.data_bytes(),
                zero_bytes: plan.zero_bytes(),
                hole_bytes: plan.hole_bytes(),
                extent_count: plan.extent_count(),
            },
            destination: &preview.destination,
            execution: &preview.execution,
            extents: preview.include_extents.then(|| Extents(plan.extents())),
        };
        serde_json::to_writer_pretty(&mut *out, &report)
            .map_err(|e| Failure::new("output", e.to_string()))?;
        writeln!(out).map_err(|e| Failure::io("write output", e))?;
    } else {
        human(preview, out).map_err(|e| Failure::io("write output", e))?;
    }
    out.flush().map_err(|e| Failure::io("flush output", e))
}
fn human(preview: &Preview, out: &mut impl Write) -> std::io::Result<()> {
    let path = preview.source.path.label();
    // Escaped path text keeps embedded control characters from changing terminal layout.
    writeln!(out, "RAW {}: {path}", preview.command)?;
    let plan = &preview.plan;
    writeln!(out, "Logical size: {} bytes", plan.logical_bytes())?;
    writeln!(
        out,
        "Extents: {} (Data {} / Zero {} / Hole {} bytes)",
        plan.extent_count(),
        plan.data_bytes(),
        plan.zero_bytes(),
        plan.hole_bytes()
    )?;
    if let Some(destination) = &preview.destination {
        writeln!(out, "Destination: {}", destination.path.label())?;
        writeln!(
            out,
            "Policy: {} ({})",
            destination.policy, destination.state
        )?;
        if destination.preserved_tail_bytes > 0 {
            writeln!(
                out,
                "Preserved destination tail: {} bytes",
                destination.preserved_tail_bytes
            )?;
        }
    }
    if let Some(execution) = &preview.execution {
        writeln!(
            out,
            "Backend requested: {}; selection: {}",
            execution.requested_backend,
            execution.selected_backend.unwrap_or("deferred")
        )?;
        writeln!(
            out,
            "Threaded payload estimate: {} bytes; budget: {} bytes; fits: {}",
            execution.threaded_payload_estimate_bytes,
            execution.memory_budget_bytes,
            execution.threaded_payload_fits_budget
        )?;
        writeln!(
            out,
            "Preview only: write access and runtime readiness are unchecked. No destination was created or written."
        )?;
    }
    if preview.include_extents {
        writeln!(out, "OFFSET\tLENGTH\tKIND")?;
        for e in plan.extents() {
            let e = extent(e);
            writeln!(out, "{}\t{}\t{}", e.offset, e.length, e.kind)?;
        }
    }
    Ok(())
}
