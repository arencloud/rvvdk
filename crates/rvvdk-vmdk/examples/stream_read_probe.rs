//! Offline/native byte qualification and read workload measurement. No guest bytes,
//! paths, source identifiers or hashes are emitted. Does not publish converted disks.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rvvdk_core::VirtualDisk;
    use rvvdk_local::LocalFileBlockDevice;
    use rvvdk_vmdk::{StreamDisk, StreamDiskLimits};
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        sync::Arc,
        time::Instant,
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 2 {
        return Err("usage: stream_read_probe IMAGE compare RAW | ranges REFERENCE TSV | sequential | random".into());
    }
    let mode = args[1].to_str().ok_or("mode")?;
    let expected = match mode {
        "compare" => 3,
        "ranges" => 4,
        "sequential" | "random" => 2,
        _ => return Err("unknown mode".into()),
    };
    if args.len() != expected {
        return Err("wrong argument count".into());
    }
    let start = Instant::now();
    let source = Arc::new(LocalFileBlockDevice::open_read_only(&args[0])?);
    let disk = StreamDisk::load(source, StreamDiskLimits::default())?;
    let admission = start.elapsed();
    let mut buffer = vec![0; 1 << 20];
    let mut reference = vec![0; 1 << 20];
    let (mut bytes, mut calls) = (0u64, 0u64);
    let started = Instant::now();
    match mode {
        "compare" | "ranges" => {
            let mut raw = File::open(&args[2])?;
            let size = raw.metadata()?.len();
            let ranges = if mode == "compare" {
                if size != disk.size() {
                    return Err("reference capacity mismatch".into());
                }
                vec![(0, 0, size)]
            } else {
                if std::fs::metadata(&args[3])?.len() > 1 << 20 {
                    return Err("range file limit".into());
                }
                let text = std::fs::read_to_string(&args[3])?;
                let mut ranges = Vec::new();
                let mut covered = 0;
                for line in text.lines() {
                    let v = line
                        .split_whitespace()
                        .map(str::parse::<u64>)
                        .collect::<Result<Vec<_>, _>>()?;
                    if v.len() != 3
                        || v[2] == 0
                        || v[1] != covered
                        || v[0].checked_add(v[2]).is_none_or(|end| end > disk.size())
                    {
                        return Err("invalid oracle range".into());
                    }
                    covered = v[1].checked_add(v[2]).ok_or("range overflow")?;
                    if covered > size {
                        return Err("reference range bounds".into());
                    }
                    ranges.push((v[0], v[1], v[2]));
                }
                if covered != size || ranges.is_empty() {
                    return Err("incomplete oracle coverage".into());
                }
                ranges
            };
            for (disk_offset, reference_offset, length) in ranges {
                raw.seek(SeekFrom::Start(reference_offset))?;
                let mut at = 0;
                while at < length {
                    let n = (length - at).min(buffer.len() as u64) as usize;
                    disk.read_exact_at(disk_offset + at, &mut buffer[..n])?;
                    raw.read_exact(&mut reference[..n])?;
                    if buffer[..n] != reference[..n] {
                        return Err("logical byte mismatch".into());
                    }
                    at += n as u64;
                    bytes += n as u64;
                    calls += 1;
                }
            }
        }
        "sequential" => {
            for grain in disk.map().grains() {
                disk.read_exact_at(grain.logical_offset(), &mut buffer[..65536])?;
                bytes += 65536;
                calls += 1;
                std::hint::black_box(&buffer[..65536]);
            }
        }
        "random" => {
            let records = disk.map().grains();
            if records.is_empty() {
                return Err("random workload needs allocated grains".into());
            }
            let mut seed = 0x511b_u64;
            for _ in 0..4096 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let g = records[(seed as usize) % records.len()];
                let within = (seed >> 32) % 61441;
                disk.read_exact_at(g.logical_offset() + within, &mut buffer[..4096])?;
                bytes += 4096;
                calls += 1;
                std::hint::black_box(&buffer[..4096]);
            }
        }
        _ => unreachable!(),
    }
    let seconds = started.elapsed().as_secs_f64();
    disk.revalidate()?;
    println!(
        "{{\"scope\":\"native_stream_reads\",\"mode\":\"{mode}\",\"capacity_bytes\":{},\"allocated_grains\":{},\"logical_bytes\":{bytes},\"read_calls\":{calls},\"admission_seconds\":{},\"read_seconds\":{seconds},\"decode_memory_bytes\":{},\"reference_matched\":{}}}",
        disk.size(),
        disk.map().grains().len(),
        admission.as_secs_f64(),
        disk.decode_memory_bytes(),
        matches!(mode, "compare" | "ranges")
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("stream_read_probe requires the Linux local-file backend");
    std::process::exit(1);
}
