use rvvdk_vmdk::{Access, CreateType, Descriptor, ErrorKind, ExtentBacking, Limits};

const MONO: &[u8] = include_bytes!("fixtures/monolithic.vmdk");
fn disk(extents: &str) -> String {
    format!("version=1\nCID=00112233\nparentCID=ffffffff\ncreateType=\"custom\"\n{extents}\n")
}
fn fails(text: &str, kind: ErrorKind) {
    assert_eq!(
        Descriptor::parse(text.as_bytes()).unwrap_err().kind,
        kind,
        "{text}"
    );
}

#[test]
fn fixtures_have_checked_contiguous_byte_ranges() {
    let mono = Descriptor::parse(MONO).unwrap();
    assert_eq!(mono.cid(), 0x1234abcd);
    assert_eq!(mono.create_type(), CreateType::MonolithicFlat);
    assert_eq!(mono.size_bytes(), 8192);
    assert_eq!(mono.metadata().len(), 4);
    assert_eq!(mono.extents()[0].access(), Access::ReadWrite);
    let split = Descriptor::parse(include_bytes!("fixtures/split.vmdk")).unwrap();
    assert_eq!(split.create_type(), CreateType::TwoGbMaxExtentFlat);
    assert_eq!(split.size_bytes(), 11 * 512);
    assert_eq!(split.extents()[1].logical_offset(), 8 * 512);
    assert_eq!(split.extents()[1].access(), Access::ReadOnly);
    let custom = Descriptor::parse(include_bytes!("fixtures/custom.vmdk")).unwrap();
    assert_eq!(custom.size_bytes(), 7 * 512);
    assert_eq!(
        custom.extents()[0].backing(),
        ExtentBacking::Flat {
            file_name: "Data #1 = café.vmdk",
            offset_bytes: 1536
        }
    );
    assert_eq!(custom.extents()[1].backing(), ExtentBacking::Zero);
    assert_eq!(custom.extents()[2].logical_offset(), 3072);
    assert_eq!(custom.extents()[2].size_bytes(), 512);
}

#[test]
fn lexical_variants_preserve_filename_case_and_borrow_input() {
    let text = " # comment\r\nVeRsIoN = 1\r\ncId=ABCDEF01\r\npArEnTcId=FFFFFFFF\r\nCrEaTeTyPe=\"2gBmAxExTeNtFlAt\" # trailing\r\nrDoNlY\t1\tflat \"UPPER café.vmdk\" 0 # ok";
    let d = Descriptor::parse(text.as_bytes()).unwrap();
    let ExtentBacking::Flat { file_name, .. } = d.extents()[0].backing() else {
        panic!()
    };
    assert_eq!(file_name, "UPPER café.vmdk");
    assert!(
        (text.as_ptr() as usize..text.as_ptr() as usize + text.len())
            .contains(&(file_name.as_ptr() as usize))
    );
}

#[test]
fn syntax_and_quoting_fail_closed() {
    for extent in [
        "RW 1 FLAT file 0",
        "RW 1 FLAT\"file\" 0",
        "RW 1 FLAT \"file\"",
        "RW 1 FLAT \"file\" 0 extra",
        "RW 1 FLAT \"file\"junk 0",
        "RW 1 ZERO \"file\"",
        "RW 1 ZERO 0",
        "RW 1 FLAT \"unterminated",
        "RW 1 FLAT \"file\" 0 a b c",
    ] {
        assert!(
            matches!(
                Descriptor::parse(disk(extent).as_bytes()).unwrap_err().kind,
                ErrorKind::Syntax(_)
            ),
            "{extent}"
        );
    }
    fails(
        &disk("RW 1 FLAT \"a\\b\" 0"),
        ErrorKind::Unsupported("quoted backslash/escape"),
    );
    fails(
        &disk("RW 1 FLAT \"\" 0"),
        ErrorKind::Layout("empty filename"),
    );
    fails(
        &disk("RW 1 ZERO").replace("version=1", "version=\"1\""),
        ErrorKind::Syntax("header value quoting"),
    );
}

#[test]
fn rejects_unsupported_variants_and_parent_chains() {
    for ty in [
        "monolithicSparse",
        "twoGbMaxExtentSparse",
        "streamOptimized",
        "vmfs",
        "vmfsThin",
        "vmfsSparse",
        "vmfsSeSparse",
        "fullDevice",
        "partitionedDevice",
        "unknown",
    ] {
        fails(
            &disk("RW 1 ZERO").replace("\"custom\"", &format!("\"{ty}\"")),
            ErrorKind::Unsupported("create type"),
        );
    }
    for ty in [
        "SPARSE",
        "VMFS",
        "VMFSSPARSE",
        "VMFSRDM",
        "VMFSRAW",
        "SESPARSE",
    ] {
        fails(
            &disk(&format!("RW 1 {ty} \"file\"")),
            ErrorKind::Unsupported("extent type"),
        );
    }
    for access in ["NOACCESS", "R", "WR"] {
        fails(
            &disk(&format!("{access} 1 ZERO")),
            ErrorKind::Unsupported("extent access"),
        );
    }
    fails(
        &disk("RW 1 ZERO").replace("parentCID=ffffffff", "parentCID=00000000"),
        ErrorKind::Unsupported("parent chain"),
    );
    fails(
        &disk("RW 1 ZERO").replace("version=1", "parentFileNameHint=\"parent\"\nversion=1"),
        ErrorKind::Unsupported("parent chain"),
    );
    fails(
        &disk("RW 1 ZERO").replace("version=1", "version=2"),
        ErrorKind::Unsupported("descriptor version"),
    );
}

#[test]
fn numbers_and_all_arithmetic_boundaries_are_checked() {
    for n in ["-1", "+1", "0x1", "1.0", "18446744073709551616"] {
        fails(&disk(&format!("RW {n} ZERO")), ErrorKind::Number);
    }
    fails(&disk("RW 0 ZERO"), ErrorKind::Layout("empty extent"));
    fails(&disk("RW 36028797018963968 ZERO"), ErrorKind::Overflow);
    fails(
        &disk("RW 1 FLAT \"x\" 36028797018963968"),
        ErrorKind::Overflow,
    );
    fails(
        &disk("RW 1 FLAT \"x\" 36028797018963967"),
        ErrorKind::Overflow,
    );
    fails(
        &disk("RW 36028797018963967 ZERO\nRW 1 ZERO"),
        ErrorKind::Overflow,
    );
    let max = disk("RW 36028797018963967 ZERO");
    assert_eq!(
        Descriptor::parse(max.as_bytes()).unwrap().size_bytes(),
        u64::MAX - 511
    );
    for cid in ["1", "123456789", "zzzzzzzz", "-1234567"] {
        fails(
            &disk("RW 1 ZERO").replace("00112233", cid),
            ErrorKind::Number,
        );
    }
}

#[test]
fn duplicates_order_and_required_fields_are_validated() {
    for field in [
        "version=1",
        "CID=00112233",
        "parentCID=ffffffff",
        "createType=\"custom\"",
    ] {
        let original = disk("RW 1 ZERO");
        assert!(matches!(
            Descriptor::parse(original.replace(&format!("{field}\n"), "").as_bytes())
                .unwrap_err()
                .kind,
            ErrorKind::Missing(_)
        ));
        fails(
            &original.replace(field, &format!("{field}\n{field}")),
            ErrorKind::Duplicate,
        );
    }
    fails(
        &disk("RW 1 ZERO").replace("CID=00112233", "CID=00112233\ncid=00112233"),
        ErrorKind::Duplicate,
    );
    fails(
        &disk("RW 1 ZERO\nversion=1"),
        ErrorKind::Syntax("header after extents"),
    );
    fails(
        &disk("RW 1 ZERO\nddb.adapterType=\"ide\"\nRW 1 ZERO"),
        ErrorKind::Syntax("extent after DDB"),
    );
    fails(
        &disk("RW 1 ZERO").replace("version=1", "ddb.adapterType=\"ide\"\nversion=1"),
        ErrorKind::Syntax("DDB before extents"),
    );
    fails(&disk(""), ErrorKind::Missing("extents"));
}

#[test]
fn metadata_is_informational_and_unknown_features_rejected() {
    fails(
        &disk("RW 1 ZERO\nddb.adapterType=\"ide\"\nDDB.ADAPTERTYPE=\"ide\""),
        ErrorKind::Duplicate,
    );
    fails(
        &disk("RW 1 ZERO\nddb.adapterType=ide"),
        ErrorKind::Syntax("DDB value must be quoted"),
    );
    for key in ["ddb.encryption", "ddb.logicalSectorSize", "ddb.unknown"] {
        fails(
            &disk(&format!("RW 1 ZERO\n{key}=\"value\"")),
            ErrorKind::Unsupported("DDB key"),
        );
    }
    for key in ["encryption.keySafe", "unknown", "isNativeSnapshot"] {
        fails(
            &disk("RW 1 ZERO").replace("version=1", &format!("{key}=\"value\"\nversion=1")),
            ErrorKind::Unsupported("header key"),
        );
    }
    let text = disk("RW 1 ZERO\nddb.geometry.sectors=\"garbage\"");
    assert_eq!(
        Descriptor::parse(text.as_bytes()).unwrap().size_bytes(),
        512
    );
}

#[test]
fn supported_create_types_enforce_shape_and_split_capacity() {
    for (extents, kind) in [
        (
            "RW 1 ZERO",
            ErrorKind::Layout("hosted flat requires FLAT at offset zero"),
        ),
        (
            "RW 1 FLAT \"x\" 1",
            ErrorKind::Layout("hosted flat requires FLAT at offset zero"),
        ),
        (
            "RW 1 FLAT \"x\" 0\nRW 1 FLAT \"y\" 0",
            ErrorKind::Layout("monolithic disk requires one extent"),
        ),
    ] {
        fails(
            &disk(extents).replace("\"custom\"", "\"monolithicFlat\""),
            kind,
        );
    }
    let split = disk("RW 4194304 FLAT \"x\" 0").replace("\"custom\"", "\"twoGbMaxExtentFlat\"");
    assert!(Descriptor::parse(split.as_bytes()).is_ok());
    fails(
        &split.replace("4194304", "4194305"),
        ErrorKind::Layout("split extent exceeds 2 GiB"),
    );
}

#[test]
fn each_resource_limit_accepts_boundary_and_rejects_next_byte_or_entry() {
    let text = disk("RW 1 FLAT \"abc\" 0\nRW 1 ZERO\nddb.uuid=\"x\"");
    let limits = Limits {
        descriptor_bytes: text.len(),
        line_bytes: text.lines().map(str::len).max().unwrap(),
        extents: 2,
        metadata_entries: 1,
        filename_bytes: 3,
    };
    assert!(Descriptor::parse_with_limits(text.as_bytes(), limits).is_ok());
    for (limited, name) in [
        (
            Limits {
                descriptor_bytes: text.len() - 1,
                ..limits
            },
            "descriptor bytes",
        ),
        (
            Limits {
                line_bytes: limits.line_bytes - 1,
                ..limits
            },
            "line bytes",
        ),
        (
            Limits {
                extents: 1,
                ..limits
            },
            "extents",
        ),
        (
            Limits {
                metadata_entries: 0,
                ..limits
            },
            "metadata entries",
        ),
        (
            Limits {
                filename_bytes: 2,
                ..limits
            },
            "filename bytes",
        ),
    ] {
        assert_eq!(
            Descriptor::parse_with_limits(text.as_bytes(), limited)
                .unwrap_err()
                .kind,
            ErrorKind::Limit(name)
        );
    }
    let max = disk(&"RW 1 ZERO\n".repeat(1024));
    assert_eq!(
        Descriptor::parse(max.as_bytes()).unwrap().extents().len(),
        1024
    );
    fails(&(max + "RW 1 ZERO"), ErrorKind::Limit("extents"));
}

#[test]
fn encoding_controls_and_error_locations_are_precise() {
    assert_eq!(
        Descriptor::parse(b"\xff").unwrap_err().kind,
        ErrorKind::Encoding
    );
    for control in ['\0', '\r', '\u{7f}', '\u{85}'] {
        fails(
            &disk(&format!("# comment{control}bad\nRW 1 ZERO")),
            ErrorKind::Encoding,
        );
    }
    fails(
        &disk("RW 1 ZERO").replace("version=1", "encoding=\"windows-1252\"\nversion=1"),
        ErrorKind::Unsupported("encoding"),
    );
    let e = Descriptor::parse(disk("RW 1 ZERO\nRW 0 ZERO").as_bytes()).unwrap_err();
    assert_eq!(e.line, 6);
    assert_eq!(
        e.to_string(),
        "VMDK descriptor line 6: invalid extent layout: empty extent"
    );
}

#[test]
fn path_policy_is_explicitly_deferred_without_any_file_access() {
    for name in [
        "/does/not/exist",
        "../outside",
        "dir/../../escape",
        "scheme:object",
    ] {
        let text = disk(&format!("RW 1 FLAT \"{name}\" 0"));
        assert_eq!(
            Descriptor::parse(text.as_bytes()).unwrap().extents()[0].backing(),
            ExtentBacking::Flat {
                file_name: name,
                offset_bytes: 0
            }
        );
    }
}

#[test]
fn deterministic_mutations_never_panic_or_escape_layout_invariants() {
    // Repeatable smoke coverage; this does not replace a future fuzzing campaign.
    let original = disk("RW 3 FLAT \"x\" 4\nRW 2 ZERO");
    for i in 0..original.len() {
        for byte in [0, b'\n', b'\r', b'"', b'=', b'#', b' ', b'9', 0xff] {
            let mut input = original.as_bytes().to_vec();
            input[i] = byte;
            if let Ok(d) = Descriptor::parse(&input) {
                let mut end = 0;
                for e in d.extents() {
                    assert_eq!(e.logical_offset(), end);
                    assert!(e.size_bytes() > 0 && e.size_bytes() % 512 == 0);
                    end = end.checked_add(e.size_bytes()).unwrap();
                }
                assert_eq!(end, d.size_bytes());
            }
        }
        let _ = Descriptor::parse(&original.as_bytes()[..i]);
    }
}
