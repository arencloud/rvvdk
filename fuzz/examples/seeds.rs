fn main() {
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("seed output directory"));
    assert!(!root.exists(), "refuse to replace existing seeds");
    for (target, name, bytes) in rvvdk_fuzz::seeds() {
        let dir = root.join(target);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), bytes).unwrap();
    }
}
