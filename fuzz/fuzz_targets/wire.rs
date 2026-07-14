#![no_main]

use libfuzzer_sys::fuzz_target;
use nickel_export::validate_cli_arguments;
use nickel_export_core::{
    ExportManifest, ExportRequest, normalize_request, verify_manifest_integrity,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(request) = serde_json::from_slice::<ExportRequest>(data) {
        let _ = normalize_request(&request);
    }
    if let Ok(manifest) = serde_json::from_slice::<ExportManifest>(data) {
        let _ = verify_manifest_integrity(manifest);
    }
    if let Ok(text) = core::str::from_utf8(data) {
        let args = text.split('\0').map(str::to_string).collect::<Vec<_>>();
        let _ = validate_cli_arguments(&args);
    }
});
