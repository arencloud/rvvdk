use super::*;
use crate::inventory::{Properties, Reference};
use crate::{About, License, xml};
use serde_json::{Value, json};

const UUID: &str = "01234567-89ab-cdef-0123-456789abcdef";
fn endpoint() -> EndpointIdentity {
    EndpointIdentity::pinned(
        "https://example.invalid",
        &"ab".repeat(32),
        PinProvenance::TrustOnFirstUse,
    )
    .unwrap()
}
fn selection() -> SourceSelection {
    SourceSelection::new(
        endpoint(),
        "vm-private",
        UUID,
        2000,
        "[store] private/disk.vmdk",
        1 << 30,
    )
    .unwrap()
}
fn observed() -> ArtifactObservation {
    ArtifactObservation {
        container_bytes: 12345,
        completeness: Completeness::Complete,
        validation: ValidationClaim::ContainerDigestVerified,
        container_sha256: Some([7; 32]),
    }
}
fn artifact() -> ExportArtifact {
    ExportArtifact::new(ArtifactId::new([1; 16]).unwrap(), &selection(), observed()).unwrap()
}
fn reference(value: &str) -> Reference {
    let raw = format!("<r type='VirtualMachine'>{value}</r>");
    Reference::parse(xml::parse(&raw).unwrap().root_element()).unwrap()
}
fn vm() -> Vm {
    let mut props = String::new();
    for (key, value) in [
        ("config.uuid", UUID),
        ("runtime.powerState", "poweredOff"),
        ("config.guestId", "otherGuest"),
        ("guest.toolsRunningStatus", "guestToolsNotRunning"),
        ("config.template", "false"),
        (
            "config.hardware.device",
            "<VirtualDevice xsi:type='VirtualDisk'><key>2000</key><capacityInKB>1048576</capacityInKB><backing xsi:type='VirtualDiskFlatVer2BackingInfo'><diskMode>persistent</diskMode><fileName>[store] private/disk.vmdk</fileName></backing></VirtualDevice>",
        ),
    ] {
        props.push_str(&format!(
            "<propSet><name>{key}</name><val>{value}</val></propSet>"
        ));
    }
    let text = format!(
        "<response xmlns='urn:vim25' xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance'><returnval><objects><obj type='VirtualMachine'>vm-private</obj>{props}</objects></returnval></response>"
    );
    let doc = xml::parse(&text).unwrap();
    let reference = reference("vm-private");
    Properties::parse(doc.root_element(), &reference)
        .unwrap()
        .vm(reference, 1, 16)
        .unwrap()
}
fn inventory(vms: Vec<Vm>) -> Inventory {
    Inventory {
        about: About {
            version: "8.0.3".into(),
            build: 1,
            api_version: "8.0.3.0".into(),
            api_type: "HostAgent",
        },
        hosts: vec![],
        datastores: vec![],
        vms,
        license: License {
            available_editions: vec![],
            active_assignment: "unresolved",
        },
    }
}
fn wire() -> Value {
    serde_json::from_slice(&artifact().to_json().unwrap()).unwrap()
}
fn parse(value: &Value) -> Result<ExportArtifact> {
    ExportArtifact::from_json(&serde_json::to_vec(value).unwrap())
}

#[test]
fn explicit_selection_rejects_same_capacity_impostors_and_duplicates() {
    let request = selection();
    assert_eq!(
        request
            .select(&endpoint(), &inventory((0..129).map(|_| vm()).collect()))
            .unwrap_err(),
        ContractError::InvalidInput
    );
    let mut other = vm();
    other.identity.reference = reference("other-vm");
    assert_eq!(
        request
            .select(&endpoint(), &inventory(vec![other]))
            .unwrap_err(),
        ContractError::Selection
    );
    assert_eq!(
        request
            .select(&endpoint(), &inventory(vec![vm(), vm()]))
            .unwrap_err(),
        ContractError::Selection
    );
    let mut unrelated = vm();
    unrelated.identity.reference = reference("unrelated");
    let inv = inventory(vec![unrelated, vm()]);
    assert_eq!(
        request.select(&endpoint(), &inv).unwrap().capacity_bytes,
        1 << 30
    );
    let mut changed = vm();
    changed.disks[0].identity.key = Some(2001);
    assert_eq!(
        request.check_vm(&endpoint(), &changed).unwrap_err(),
        ContractError::IdentityMismatch
    );
    changed = vm();
    changed.disks[0].identity.backing = Some("[store] replaced.vmdk".into());
    assert_eq!(
        request.check_vm(&endpoint(), &changed).unwrap_err(),
        ContractError::IdentityMismatch
    );
    let other_uuid = SourceSelection::new(
        endpoint(),
        "vm-private",
        "11234567-89ab-cdef-0123-456789abcdef",
        2000,
        "[store] private/disk.vmdk",
        1 << 30,
    )
    .unwrap();
    assert_eq!(
        other_uuid.check_vm(&endpoint(), &vm()).unwrap_err(),
        ContractError::IdentityMismatch
    );
}

#[test]
fn v1_golden_binding_and_explicit_inventory_access_remain_stable() {
    // Independently generated using SHA-256 and little-endian u64 field lengths.
    let decoded = ExportArtifact::from_json(include_bytes!("artifact-v1.json")).unwrap();
    assert_eq!(decoded, artifact());
    assert_eq!(decoded.check_source(&selection()), Ok(()));
    let v = vm();
    assert_eq!(v.disks[0].device_key(), Some(2000));
    assert_eq!(
        v.disks[0].backing_identity(),
        Some("[store] private/disk.vmdk")
    );
    let diagnostic = serde_json::to_string(&v.disks[0]).unwrap();
    assert!(!diagnostic.contains("2000"));
    assert!(!diagnostic.contains("private/disk.vmdk"));
}

#[test]
fn identity_check_retains_power_topology_and_backing_scope_gates() {
    let request = selection();
    let mutations: &[fn(&mut Vm)] = &[
        |v| v.power_state = "poweredOn".into(),
        |v| v.template = true,
        |v| v.snapshot_present = true,
        |v| v.export_disabled_method_list = true,
        |v| v.disks[0].capacity_bytes += 512,
        |v| v.disks[0].encrypted = true,
        |v| v.disks[0].parent_present = true,
        |v| v.disks[0].disk_mode = "independent_persistent".into(),
        |v| v.disks[0].backing_type = "unqualified".into(),
        |v| v.disks[0].identity.key = None,
        |v| v.disks[0].identity.backing = None,
        |v| v.disks.clear(),
        |v| v.disks.push(vm().disks.remove(0)),
    ];
    for mutate in mutations {
        let mut v = vm();
        mutate(&mut v);
        assert_eq!(
            request.check_vm(&endpoint(), &v).unwrap_err(),
            ContractError::UnsupportedScope
        );
    }
    let mut v = vm();
    v.label = "different display label".into();
    assert!(request.check_vm(&endpoint(), &v).is_ok());
}

#[test]
fn trust_endpoint_and_source_components_are_bound() {
    let artifact = artifact();
    for ep in [
        EndpointIdentity::pinned(
            "https://other.invalid",
            &"ab".repeat(32),
            PinProvenance::TrustOnFirstUse,
        )
        .unwrap(),
        EndpointIdentity::pinned(
            "https://example.invalid:8443",
            &"ab".repeat(32),
            PinProvenance::TrustOnFirstUse,
        )
        .unwrap(),
        EndpointIdentity::pinned(
            "https://example.invalid",
            &"cd".repeat(32),
            PinProvenance::TrustOnFirstUse,
        )
        .unwrap(),
        EndpointIdentity::pinned(
            "https://example.invalid",
            &"ab".repeat(32),
            PinProvenance::ExternallyVerified,
        )
        .unwrap(),
    ] {
        assert_eq!(
            selection().check_vm(&ep, &vm()).unwrap_err(),
            ContractError::IdentityMismatch
        );
        let other = SourceSelection::new(
            ep,
            "vm-private",
            UUID,
            2000,
            "[store] private/disk.vmdk",
            1 << 30,
        )
        .unwrap();
        assert_eq!(
            artifact.check_source(&other),
            Err(ContractError::IdentityMismatch)
        );
    }
    for (reference, uuid, key, backing, bytes) in [
        ("other", UUID, 2000, "[store] private/disk.vmdk", 1 << 30),
        (
            "vm-private",
            "11234567-89ab-cdef-0123-456789abcdef",
            2000,
            "[store] private/disk.vmdk",
            1 << 30,
        ),
        (
            "vm-private",
            UUID,
            2001,
            "[store] private/disk.vmdk",
            1 << 30,
        ),
        ("vm-private", UUID, 2000, "other backing", 1 << 30),
        (
            "vm-private",
            UUID,
            2000,
            "[store] private/disk.vmdk",
            2 << 30,
        ),
    ] {
        let other = SourceSelection::new(endpoint(), reference, uuid, key, backing, bytes).unwrap();
        assert_eq!(
            artifact.check_source(&other),
            Err(ContractError::IdentityMismatch)
        );
    }
}

#[test]
fn canonical_endpoint_pin_and_uuid_spellings_match() {
    let ep = EndpointIdentity::pinned(
        "https://EXAMPLE.invalid:443/sdk",
        &"AB".repeat(32),
        PinProvenance::TrustOnFirstUse,
    )
    .unwrap();
    let same = SourceSelection::new(
        ep,
        "vm-private",
        &UUID.to_ascii_uppercase(),
        2000,
        "[store] private/disk.vmdk",
        1 << 30,
    )
    .unwrap();
    assert_eq!(artifact().check_source(&same), Ok(()));
    assert_ne!(bind(&[b"ab", b"c"]), bind(&[b"a", b"bc"]));
}

#[test]
fn round_trip_preserves_claims_but_never_skips_fresh_container_checks() {
    for validation in [
        ValidationClaim::Unchecked,
        ValidationClaim::ContainerDigestVerified,
        ValidationClaim::LogicalReadbackVerified,
    ] {
        let mut observation = observed();
        observation.validation = validation;
        let original =
            ExportArtifact::new(ArtifactId::new([1; 16]).unwrap(), &selection(), observation)
                .unwrap();
        let decoded = ExportArtifact::from_json(&original.to_json().unwrap()).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.check_source(&selection()), Ok(()));
        assert_eq!(decoded.check_container(12345, &[7; 32]), Ok(()));
        assert_eq!(
            decoded.check_container(12346, &[7; 32]),
            Err(ContractError::ContentMismatch)
        );
        assert_eq!(
            decoded.check_container(12345, &[8; 32]),
            Err(ContractError::ContentMismatch)
        );
    }
    let mut observation = observed();
    observation.container_bytes = 2 << 30;
    assert!(
        ExportArtifact::new(ArtifactId::new([1; 16]).unwrap(), &selection(), observation).is_ok()
    );
}

#[test]
fn incomplete_artifacts_cannot_claim_checks_or_complete_content() {
    let id = ArtifactId::new([1; 16]).unwrap();
    for bytes in [0, 12345, MAX_CONTAINER_BYTES] {
        let observation = ArtifactObservation {
            container_bytes: bytes,
            completeness: Completeness::Incomplete,
            validation: ValidationClaim::Unchecked,
            container_sha256: None,
        };
        let original = ExportArtifact::new(id, &selection(), observation.clone()).unwrap();
        let decoded = ExportArtifact::from_json(&original.to_json().unwrap()).unwrap();
        assert_eq!(original, decoded);
        assert_eq!(
            decoded.check_container(bytes, &[7; 32]),
            Err(ContractError::Incomplete)
        );
        let mut bad = observation.clone();
        bad.validation = ValidationClaim::LogicalReadbackVerified;
        assert!(ExportArtifact::new(id, &selection(), bad).is_err());
        let mut bad = observation;
        bad.container_sha256 = Some([7; 32]);
        assert!(ExportArtifact::new(id, &selection(), bad).is_err());
    }
}

#[test]
fn malformed_versions_sizes_digests_states_and_extensions_fail_closed() {
    let fields = [
        ("schema_version", json!(2)),
        ("schema_version", json!(-1)),
        ("artifact_id", json!("0".repeat(32))),
        ("artifact_id", json!("f".repeat(31))),
        ("source_binding", json!("z".repeat(64))),
        ("source_binding", json!("A".repeat(64))),
        ("logical_bytes", json!(0)),
        ("logical_bytes", json!(513)),
        ("logical_bytes", json!(MAX_LOGICAL_BYTES + 512)),
        ("logical_bytes", json!(1.5)),
        ("container_bytes", json!(0)),
        ("container_bytes", json!(MAX_CONTAINER_BYTES + 1)),
        ("container_sha256", Value::Null),
        ("container_sha256", json!("f".repeat(65))),
        ("completeness", json!("incomplete")),
        ("validation", json!("trusted")),
        ("pin_provenance", json!("insecure")),
        ("container_format", json!("raw")),
        ("lease_reference", json!("private-lease")),
        ("path", json!("../disk")),
        ("cookie", json!("private-cookie")),
    ];
    for (key, value) in fields {
        let mut w = wire();
        w[key] = value;
        assert!(parse(&w).is_err(), "{key}");
    }
    let mut w = wire();
    w["schema_version"] = json!(2);
    assert_eq!(parse(&w).unwrap_err(), ContractError::Version);
    for key in [
        "schema_version",
        "artifact_id",
        "source_binding",
        "logical_bytes",
        "container_bytes",
        "completeness",
        "validation",
        "pin_provenance",
        "container_format",
    ] {
        let mut w = wire();
        w.as_object_mut().unwrap().remove(key);
        assert!(parse(&w).is_err());
    }
    let mut w = wire();
    w["pin_provenance"] = json!("externally_verified");
    assert_eq!(
        parse(&w).unwrap().check_source(&selection()),
        Err(ContractError::IdentityMismatch)
    );
}

#[test]
fn parser_budget_duplicates_nesting_overflow_and_trailing_data_are_rejected() {
    let bytes = artifact().to_json().unwrap();
    for invalid in [
        vec![b' '; MAX_METADATA_BYTES + 1],
        vec![255],
        b"{\"schema_version\":18446744073709551616}".to_vec(),
        [b"{\"schema_version\":1,".as_slice(), &bytes[1..]].concat(),
        [bytes.as_slice(), b" extra"].concat(),
        [vec![b'['; 150], vec![b']'; 150]].concat(),
    ] {
        let err = ExportArtifact::from_json(&invalid).unwrap_err();
        assert_eq!(err, ContractError::InvalidInput);
    }
    let mut padded = bytes.clone();
    padded.resize(MAX_METADATA_BYTES, b' ');
    assert!(ExportArtifact::from_json(&padded).is_ok());
    padded.push(b' ');
    assert!(ExportArtifact::from_json(&padded).is_err());
    for n in 0..bytes.len() {
        assert!(ExportArtifact::from_json(&bytes[..n]).is_err());
    }
}

#[test]
fn runtime_input_bounds_and_redaction() {
    for reference in ["".into(), "x".repeat(257), "bad\nreference".into()] {
        assert!(SourceSelection::new(endpoint(), &reference, UUID, 2000, "backing", 512).is_err());
    }
    for uuid in ["-".repeat(36), "0".repeat(36), "invalid".into()] {
        assert!(SourceSelection::new(endpoint(), "vm", &uuid, 2000, "backing", 512).is_err());
    }
    for key in [0, i32::MAX as u64 + 1] {
        assert!(SourceSelection::new(endpoint(), "vm", UUID, key, "backing", 512).is_err());
    }
    for backing in [String::new(), "x".repeat(4097), "bad\0backing".into()] {
        assert!(SourceSelection::new(endpoint(), "vm", UUID, 2000, &backing, 512).is_err());
    }
    for url in [
        "http://example.invalid",
        "https://u:p@example.invalid",
        "https://example.invalid/?ticket=x",
    ] {
        assert!(
            EndpointIdentity::pinned(url, &"ab".repeat(32), PinProvenance::TrustOnFirstUse)
                .is_err()
        );
    }
    assert!(
        SourceSelection::new(
            endpoint(),
            &"v".repeat(256),
            UUID,
            i32::MAX as u64,
            &"b".repeat(4096),
            MAX_LOGICAL_BYTES
        )
        .is_ok()
    );
    let a = artifact();
    let debug = format!(
        "{:?}{:?}{:?}{:?}{:?}",
        a,
        a.id(),
        selection(),
        endpoint(),
        observed()
    );
    let serialized = String::from_utf8(a.to_json().unwrap()).unwrap();
    for private in [
        UUID,
        "vm-private",
        "example.invalid",
        "private/disk.vmdk",
        "abababab",
        "07070707",
    ] {
        assert!(!debug.contains(private));
        if private != "07070707" {
            assert!(!serialized.contains(private));
        }
    }
    assert!(serialized.contains("07070707")); // Content digests are intentionally private persistence.
}
