use super::*;
use crate::recovery::tests::Temp;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};
pub(in crate::recovery) fn valid() -> Value {
    json!({"schema_version":1,"endpoint":"https://example.invalid","certificate_sha256":"ab".repeat(32),"pin_provenance":"trust_on_first_use","selection":{"vm_reference":"vm-synthetic","bios_uuid":"01234567-89ab-cdef-0123-456789abcdef","disk_key":2000,"backing":"synthetic","logical_bytes":2097152},"store":"/synthetic/store","destination":"/synthetic/destination","artifact_id":"11".repeat(16),"output_id":"17".repeat(16)})
}
fn rejects(bytes: &[u8]) {
    assert!(matches!(parse(bytes), Err(Status::Invalid)));
}
#[test]
fn strict_typed_schema_rejects_missing_duplicate_unknown_null_and_trailing_input() {
    let mut good = valid();
    good["timeout_seconds"] = json!(3600);
    assert!(parse(&serde_json::to_vec(&good).unwrap()).is_ok());
    for key in good
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| *k != "timeout_seconds")
    {
        let mut v = good.clone();
        v.as_object_mut().unwrap().remove(key);
        rejects(&serde_json::to_vec(&v).unwrap());
    }
    for key in good["selection"].as_object().unwrap().keys() {
        let mut v = good.clone();
        v["selection"].as_object_mut().unwrap().remove(key);
        rejects(&serde_json::to_vec(&v).unwrap());
    }
    for key in ["password", "user", "max_encoded_bytes", "operation"] {
        let mut v = good.clone();
        v[key] = json!("private-sentinel");
        rejects(&serde_json::to_vec(&v).unwrap());
    }
    let mut v = good.clone();
    v["selection"]["unknown"] = json!(0);
    rejects(&serde_json::to_vec(&v).unwrap());
    for key in ["schema_version", "endpoint", "selection", "timeout_seconds"] {
        let text = serde_json::to_string(&good).unwrap();
        let insert = format!(
            "\"{key}\":{},",
            if key == "timeout_seconds" {
                json!(3600)
            } else {
                good[key].clone()
            }
        );
        rejects(format!("{{{insert}{}", &text[1..]).as_bytes());
    }
    let text = serde_json::to_string(&good).unwrap();
    rejects(
        text.replace("\"disk_key\":2000", "\"disk_key\":2000,\"disk_key\":2000")
            .as_bytes(),
    );
    for bytes in [
        b"".as_slice(),
        b"{}",
        b"\xff",
        format!("{text} {{}}").as_bytes(),
        format!("{text} bad").as_bytes(),
    ] {
        rejects(bytes);
    }
    let mut v = good;
    v["timeout_seconds"] = Value::Null;
    rejects(&serde_json::to_vec(&v).unwrap());
}
#[test]
fn constructors_enforce_ids_identity_paths_and_timeout() {
    for (field, values) in [
        ("schema_version", vec![json!(0), json!(2), json!("1")]),
        (
            "timeout_seconds",
            vec![json!(0), json!(21601), json!(-1), json!(1.5), json!("3600")],
        ),
        (
            "artifact_id",
            vec![
                json!("0".repeat(32)),
                json!("zz".repeat(16)),
                json!("x"),
                json!("é".repeat(16)),
            ],
        ),
        (
            "output_id",
            vec![json!("0".repeat(32)), json!("aa".repeat(17))],
        ),
        (
            "store",
            vec![json!("relative"), json!(""), json!("/a\u{0}b")],
        ),
        ("destination", vec![json!("relative")]),
        (
            "endpoint",
            vec![
                json!("http://example.invalid"),
                json!("https://user:password@example.invalid"),
                json!("x".repeat(2049)),
            ],
        ),
        (
            "certificate_sha256",
            vec![json!("ab"), json!("gg".repeat(32))],
        ),
        ("pin_provenance", vec![json!("unknown"), Value::Null]),
    ] {
        for value in values {
            let mut v = valid();
            v[field] = value;
            rejects(&serde_json::to_vec(&v).unwrap());
        }
    }
    for (field, value) in [
        ("vm_reference", json!("")),
        ("backing", json!("x".repeat(4097))),
        ("disk_key", json!(0)),
        ("logical_bytes", json!(513)),
        ("bios_uuid", json!("bad")),
    ] {
        let mut v = valid();
        v["selection"][field] = value;
        rejects(&serde_json::to_vec(&v).unwrap());
    }
    for value in [1, 21600] {
        let mut v = valid();
        v["timeout_seconds"] = json!(value);
        v["pin_provenance"] = json!("externally_verified");
        v["artifact_id"] = json!("AB".repeat(16));
        assert!(parse(&serde_json::to_vec(&v).unwrap()).is_ok());
    }
}
#[test]
fn bounded_private_file_admission_and_nonblocking_special_files() {
    let mut bytes = serde_json::to_vec(&valid()).unwrap();
    bytes.resize(16384, b' ');
    let t = Temp::new();
    let p = t.file(&bytes);
    assert!(read(&p).is_ok());
    fs::write(&p, [bytes.as_slice(), b" "].concat()).unwrap();
    assert!(matches!(read(&p), Err(Status::Failed)));
    fs::write(&p, &bytes).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o640)).unwrap();
    assert!(matches!(read(&p), Err(Status::Failed)));
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    let link = t.0.join("link");
    symlink(&p, &link).unwrap();
    assert!(matches!(read(&link), Err(Status::Failed)));
    fs::remove_file(link).unwrap();
    let hard = t.0.join("hard");
    fs::hard_link(&p, &hard).unwrap();
    assert!(matches!(read(&p), Err(Status::Failed)));
    fs::remove_file(hard).unwrap();
    assert!(matches!(read(&t.0), Err(Status::Failed)));
    let fifo = t.0.join("fifo");
    let name = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: valid NUL-terminated fixture path.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(matches!(read(&fifo), Err(Status::Failed)));
    assert!(matches!(read(Path::new("/dev/null")), Err(Status::Failed)));
    let socket = t.0.join("socket");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    assert!(matches!(read(&socket), Err(Status::Failed)));
    let before = fs::metadata(&p).unwrap();
    fs::write(&p, b"{}").unwrap();
    assert!(!same(&before, &fs::metadata(&p).unwrap()));
}
