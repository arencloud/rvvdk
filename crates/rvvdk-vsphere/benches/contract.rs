use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_vsphere::contract::*;
use std::{hint::black_box, time::Duration};

fn benchmarks(c: &mut Criterion) {
    let pin = "ab".repeat(32);
    let endpoint = EndpointIdentity::pinned(
        "https://example.invalid",
        &pin,
        PinProvenance::TrustOnFirstUse,
    )
    .unwrap();
    let reference = "v".repeat(256);
    let backing = "b".repeat(4096);
    let uuid = "01234567-89ab-cdef-0123-456789abcdef";
    let source = SourceSelection::new(
        endpoint.clone(),
        &reference,
        uuid,
        2000,
        &backing,
        MAX_LOGICAL_BYTES,
    )
    .unwrap();
    let complete = ExportArtifact::new(
        ArtifactId::new([1; 16]).unwrap(),
        &source,
        ArtifactObservation {
            container_bytes: MAX_CONTAINER_BYTES,
            completeness: Completeness::Complete,
            validation: ValidationClaim::ContainerDigestVerified,
            container_sha256: Some([7; 32]),
        },
    )
    .unwrap();
    let incomplete = ExportArtifact::new(
        ArtifactId::new([2; 16]).unwrap(),
        &source,
        ArtifactObservation {
            container_bytes: 0,
            completeness: Completeness::Incomplete,
            validation: ValidationClaim::Unchecked,
            container_sha256: None,
        },
    )
    .unwrap();
    let encoded = complete.to_json().unwrap();
    let partial = incomplete.to_json().unwrap();
    let over_budget = vec![b' '; MAX_METADATA_BYTES + 1];
    let mut padded = encoded.clone();
    padded.resize(MAX_METADATA_BYTES, b' ');
    let mut group = c.benchmark_group("export_contract");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(30);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.bench_function("endpoint", |b| {
        b.iter(|| {
            EndpointIdentity::pinned(
                black_box("https://example.invalid"),
                black_box(&pin),
                PinProvenance::TrustOnFirstUse,
            )
            .unwrap()
        })
    });
    group.bench_function("identity_max", |b| {
        b.iter(|| {
            SourceSelection::new(
                black_box(endpoint.clone()),
                black_box(&reference),
                black_box(uuid),
                2000,
                black_box(&backing),
                MAX_LOGICAL_BYTES,
            )
            .unwrap()
        })
    });
    group.bench_function("parse_complete", |b| {
        b.iter(|| ExportArtifact::from_json(black_box(&encoded)).unwrap())
    });
    group.bench_function("parse_incomplete", |b| {
        b.iter(|| ExportArtifact::from_json(black_box(&partial)).unwrap())
    });
    group.bench_function("parse_max_bytes", |b| {
        b.iter(|| ExportArtifact::from_json(black_box(&padded)).unwrap())
    });
    group.bench_function("encode_complete", |b| {
        b.iter(|| black_box(&complete).to_json().unwrap())
    });
    group.bench_function("check_binding", |b| {
        b.iter(|| {
            black_box(&complete)
                .check_source(black_box(&source))
                .unwrap()
        })
    });
    group.bench_function("reject_over_budget", |b| {
        b.iter(|| ExportArtifact::from_json(black_box(&over_budget)).unwrap_err())
    });
    group.finish();
}
criterion_group!(benches, benchmarks);
criterion_main!(benches);
