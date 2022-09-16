#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use makepad_regex::Regex;

    if let Ok(pattern) = std::str::from_utf8(data) {
        let _ = Regex::new(pattern);
    }
});
