#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = rvvdk_fuzz::descriptor(data);
});
