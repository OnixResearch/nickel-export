//! Positive and hostile controls for the new mechanisms.
#![allow(
    clippy::unwrap_used,
    reason = "nickel-export test owner: failed fixture setup or positive assertion must fail this private test module"
)]
use super::*;
use std::time::Instant;
const TEST_TIMEOUT_MILLIS: u64 = 100;
const TEST_UPPER_SECONDS: u64 = 3;
const TEST_STREAM_BYTES: u64 = 64;
const EXECUTABLE_MODE: u32 = 0o700;
const EXPECTED_VERSION: &str = "1.17.0";
const LARGE_BUFFER_CHUNKS: usize = 3;
const LARGE_BYTES: usize = STREAM_BUFFER_BYTES * LARGE_BUFFER_CHUNKS;
const BENCH_REPETITIONS: usize = 32;
const BENCH_INPUT_BYTES: usize = 1_048_576;

fn snapshot() -> EvaluationSnapshot {
    EvaluationSnapshot {
        root: create_snapshot_root().unwrap(),
    }
}
#[cfg(unix)]
fn program(root: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let shell = resolve_evaluator_program(Path::new("sh")).unwrap();
    let path = root.join("evaluator");
    fs::write(&path, format!("#!{}\n{body}\n", shell.display())).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(EXECUTABLE_MODE)).unwrap();
    path
}
fn limits() -> ResourceLimits {
    ResourceLimits {
        evaluator_timeout_milliseconds: TEST_TIMEOUT_MILLIS,
        max_artifact_bytes: TEST_STREAM_BYTES,
        max_stderr_bytes: TEST_STREAM_BYTES,
        ..ResourceLimits::DEFAULT
    }
}

#[cfg(unix)]
// r[verify nickel_export.improvements.process]
#[test]
fn version_probe_is_bounded_and_does_not_disclose_output() {
    let root = snapshot();
    let executable = program(&root.root, "printf 1.17.0");
    assert!(verify_evaluator_version(&executable, EXPECTED_VERSION, &limits()).is_ok());
    assert!(verify_evaluator_version(&executable, "missing", &limits()).is_err());
    for script in [
        "while :; do :; done",
        "while :; do printf 0123456789; done",
        "while :; do printf 0123456789 >&2; done",
        "printf CANARY-private-value",
    ] {
        let executable = program(&root.root, script);
        let started = Instant::now();
        let error = verify_evaluator_version(&executable, EXPECTED_VERSION, &limits()).unwrap_err();
        assert!(!error.to_string().contains("CANARY-private-value"));
        assert!(started.elapsed() < Duration::from_secs(TEST_UPPER_SECONDS));
    }
}

// r[verify nickel_export.improvements.memory]
#[test]
fn streaming_hash_matches_exact_bytes_and_rejects_oversize() {
    let root = snapshot();
    let path = root.root.join("artifact");
    for content in [Vec::new(), vec![1; LARGE_BYTES]] {
        fs::write(&path, &content).unwrap();
        assert_eq!(
            evaluator_artifact_identity(&path, u64::try_from(content.len()).unwrap()).unwrap(),
            blake3_identity(&content)
        );
    }
    assert!(evaluator_artifact_identity(&path, 1).is_err());
}

// r[verify nickel_export.improvements.aggregate]
#[test]
fn aggregate_admission_checks_before_retaining_and_rejects_overflow() {
    let root = snapshot();
    let content = b"true";
    fs::write(root.root.join("source"), content).unwrap();
    let limits = ResourceLimits {
        max_input_bytes: u64::try_from(content.len()).unwrap(),
        ..ResourceLimits::DEFAULT
    };
    let mut retained = 0;
    assert!(read_budgeted_input(&root.root, "source", "test", &limits, &mut retained).is_ok());
    assert!(read_budgeted_input(&root.root, "source", "test", &limits, &mut retained).is_err());
    assert_eq!(retained, limits.max_input_bytes);
    assert_eq!(limits.checked_input_total(u64::MAX, 1), None);
}

#[test]
fn capture_borrows_bytes_and_replay_keeps_the_original_reference() {
    let mut request: ExportRequest =
        serde_json::from_str(include_str!("../../../fixtures/requests/json.json")).unwrap();
    request.dependencies.clear();
    let content = vec![1; LARGE_BYTES];
    let capture = capture_files(&request, &content, &[]).unwrap();
    assert!(std::ptr::eq(capture[0].bytes.as_ptr(), content.as_ptr()));
    request.dependencies.push("missing".into());
    assert!(capture_files(&request, &content, &[]).is_err());
    let pointer = content.as_ptr();
    let mut first = Some(content);
    let mut runs = 0;
    let profile = ReplayProfile {
        requested_runs: REPLAY_MINIMUM_RUNS,
        maximum_runs: REPLAY_MINIMUM_RUNS,
    };
    let result = execute_replay_runs(profile.clone(), "plan", "artifact", "limits", || {
        runs += 1;
        Ok(first.take().unwrap_or_else(|| vec![1; LARGE_BYTES]))
    })
    .unwrap();
    assert_eq!(runs, profile.requested_runs);
    assert_eq!(result.agreed_output.as_ref().unwrap().as_ptr(), pointer);
    assert!(assess_replay(profile, "plan", "artifact", "limits", &[]).is_err());
}

#[test]
#[ignore = "explicit release-mode measurement, not a timing acceptance test"]
// r[verify nickel_export.improvements.measurement]
fn measure_capture_and_replay() {
    let mut request: ExportRequest =
        serde_json::from_str(include_str!("../../../fixtures/requests/json.json")).unwrap();
    request.dependencies.clear();
    let input = vec![1; BENCH_INPUT_BYTES];
    let started = Instant::now();
    for _ in 0..BENCH_REPETITIONS {
        std::hint::black_box(input.clone());
    }
    let old_nanos = started.elapsed().as_nanos();
    let started = Instant::now();
    for _ in 0..BENCH_REPETITIONS {
        std::hint::black_box(capture_files(&request, &input, &[]).unwrap());
    }
    let new_nanos = started.elapsed().as_nanos();
    println!(
        "capture,input_bytes={BENCH_INPUT_BYTES},runs={BENCH_REPETITIONS},copy_ns={old_nanos},borrow_ns={new_nanos}"
    );
    let profile = ReplayProfile {
        requested_runs: ResourceLimits::DEFAULT.max_replay_runs,
        maximum_runs: ResourceLimits::DEFAULT.max_replay_runs,
    };
    let assessment = execute_replay_runs(profile.clone(), "plan", "artifact", "limits", || {
        Ok(input.clone())
    })
    .unwrap();
    assert_eq!(assessment.agreed_output.unwrap(), input);
    println!(
        "replay,runs={},retained_reference_bytes={BENCH_INPUT_BYTES},maximum_live_output_buffers=2",
        profile.requested_runs
    );
}
