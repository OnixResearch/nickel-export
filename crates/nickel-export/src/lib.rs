//! Thin std shell for deterministic Nickel exports.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use nickel_export_core::{
    ArtifactMaterial, EvaluationObservation, EvaluatorDescriptor, ExportFormat, ExportManifest,
    ExportRequest, ImportPathPolicy, ResourceLimits, VerifiedManifest, admit_manifest,
    blake3_identity, build_manifest, build_receipt, normalize_request, verify_manifest_fresh,
    verify_manifest_integrity, verify_supplied_artifacts,
};

/// Stable non-zero process exit used for all fail-closed shell errors.
pub const FAILURE_EXIT_CODE: i32 = 2;
/// Structured shell failure schema.
pub const SHELL_ERROR_SCHEMA: &str = "onix-nickel-export-shell-error/v1";
const USAGE: &str = "usage: nickel-export export --spec <request.json> --root <dir> --evaluator <program> --evaluator-identity <identity> --evaluator-version <version> --manifest <relative-path> [--replay-runs <count>] (--write|--check)";
const VERIFY_USAGE: &str =
    "usage: nickel-export verify --manifest <relative-path> --root <dir> [--check-artifacts]";
const COMMAND_EXPORT: &str = "export";
const COMMAND_VERIFY: &str = "verify";
const FIRST_OPTION_INDEX: usize = 2;
const FLAG_SPEC: &str = "--spec";
const FLAG_ROOT: &str = "--root";
const FLAG_EVALUATOR: &str = "--evaluator";
const FLAG_EVALUATOR_IDENTITY: &str = "--evaluator-identity";
const FLAG_EVALUATOR_VERSION: &str = "--evaluator-version";
const FLAG_MANIFEST: &str = "--manifest";
const FLAG_WRITE: &str = "--write";
const FLAG_CHECK: &str = "--check";
const FLAG_CHECK_ARTIFACTS: &str = "--check-artifacts";
const FLAG_REPLAY_RUNS: &str = "--replay-runs";
const NICKEL_PACKAGE_VERSION_PREFIX: &str = "nickel-lang-cli-";
const REPLAY_REPORT_SCHEMA: &str = "onix-nickel-export-replay-report/v1";
const REPLAY_MINIMUM_RUNS: usize = 2;
const REPLAY_NON_CLAIM: &str = "Replay agreement applies only to the selected sequential runs under the recorded captured plan, evaluator artifact, and resource profile; it does not prove future or universal determinism";
const SNAPSHOT_DIRECTORY_PREFIX: &str = "nickel-export-snapshot";
const SNAPSHOT_PACKAGE_CACHE: &str = ".nickel-package-cache";
const SNAPSHOT_CREATE_ATTEMPTS: u64 = 64;
const STREAM_BUFFER_BYTES: usize = 8_192;
const MATERIALIZATION_LOCK_PATH: &str = ".nickel-export.lock";
const TRANSACTION_MARKER_PATH: &str = ".nickel-export.transaction.json";
const STAGED_FILE_TAG: &str = "nickel-export-tmp";
const LOCK_ACQUIRE_ATTEMPTS: u64 = 2;
const DEFAULT_RESOURCE_LIMITS_JSON: &str =
    include_str!("../../../config/generated/resource-limits.json");
static SNAPSHOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static MATERIALIZATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Shell error with a stable stage classification.
#[derive(Debug, Serialize)]
pub struct ShellError {
    schema: &'static str,
    stage: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    replay: Option<Box<ReplayReport>>,
}

impl ShellError {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            schema: SHELL_ERROR_SCHEMA,
            stage,
            message: message.into(),
            replay: None,
        }
    }

    fn with_replay(mut self, replay: ReplayReport) -> Self {
        self.replay = Some(Box::new(replay));
        self
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.stage, self.message)
    }
}

impl std::error::Error for ShellError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Write,
    Check,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CliOptions {
    spec: PathBuf,
    root: PathBuf,
    evaluator: PathBuf,
    evaluator_identity: String,
    evaluator_version: String,
    manifest: PathBuf,
    replay_runs: Option<usize>,
    mode: Mode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifyOptions {
    root: PathBuf,
    manifest: PathBuf,
    check_artifacts: bool,
}

#[derive(Debug, Serialize)]
struct IntegrityReport {
    schema: &'static str,
    manifest_identity: String,
    artifacts_checked: usize,
    claim: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReplayVerdict {
    Agreement,
    Divergence,
    Failure,
}

impl ReplayVerdict {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Agreement => "agreement",
            Self::Divergence => "divergence",
            Self::Failure => "failure",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReplayRunStatus {
    Success,
    Failure,
}

impl ReplayRunStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReplayProfile {
    requested_runs: usize,
    maximum_runs: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReplayRunOutcome {
    run: usize,
    status: ReplayRunStatus,
    output_identity: String,
    output_bytes: u64,
    failure_stage: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReplayReport {
    schema: &'static str,
    profile: ReplayProfile,
    plan_identity: String,
    evaluator_artifact_identity: String,
    resource_profile_identity: String,
    outcomes: Vec<ReplayRunOutcome>,
    verdict: ReplayVerdict,
    report_identity: String,
    non_claim: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReplayAttempt {
    Success(Vec<u8>),
    Failure(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayAssessment {
    report: ReplayReport,
    agreed_output: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransactionArtifact {
    temporary: String,
    destination: String,
    identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MaterializationTransaction {
    schema: String,
    output: TransactionArtifact,
    manifest: TransactionArtifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecoveryAction {
    PublishTemporary,
    DestinationAlreadyPublished,
}

#[derive(Debug)]
struct MaterializationLock {
    path: PathBuf,
}

impl Drop for MaterializationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct CanonicalEvaluationPlan {
    source: String,
    import_paths: Vec<String>,
    selector: String,
    contract: String,
    format: String,
    color: String,
    package_cache_policy: String,
    resource_profile_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvaluationPlan {
    program: PathBuf,
    args: Vec<OsString>,
    current_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapturedFile {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct EvaluationSnapshot {
    root: PathBuf,
}

impl Drop for EvaluationSnapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug)]
struct LoadedExport {
    root: PathBuf,
    request: ExportRequest,
    source_bytes: Vec<u8>,
    dependency_bytes: Vec<(String, Vec<u8>)>,
}

#[derive(Debug)]
struct EvaluatedExport {
    output_bytes: Vec<u8>,
    evaluator: EvaluatorDescriptor,
    replay: Option<ReplayReport>,
}

/// Parse arbitrary CLI arguments without performing side effects.
///
/// This is a fuzzing and embedding seam; `true` means one supported command
/// parsed completely, not that its referenced files or evaluator are valid.
#[doc(hidden)]
#[must_use]
pub fn validate_cli_arguments(args: &[String]) -> bool {
    if args.len() > ResourceLimits::DEFAULT.max_artifacts {
        return false;
    }
    match args.get(1).map(String::as_str) {
        Some(COMMAND_EXPORT) => parse_args(args).is_ok(),
        Some(COMMAND_VERIFY) => parse_verify_args(args).is_ok(),
        _ => false,
    }
}

/// Execute one supported CLI command.
///
/// # Errors
///
/// Returns a staged error for invalid arguments, unsafe or unavailable files,
/// evaluator failure, evidence rejection, verification failure, lock
/// contention, or materialization failure.
pub fn run(args: &[String]) -> Result<(), ShellError> {
    match args.get(1).map(String::as_str) {
        Some(COMMAND_EXPORT) => execute(&parse_args(args)?),
        Some(COMMAND_VERIFY) => execute_verify(&parse_verify_args(args)?),
        _ => Err(ShellError::new(
            "arguments",
            format!("{USAGE}; {VERIFY_USAGE}"),
        )),
    }
}

fn resource_limits() -> Result<ResourceLimits, ShellError> {
    let limits: ResourceLimits = serde_json::from_str(DEFAULT_RESOURCE_LIMITS_JSON)
        .map_err(|error| ShellError::new("resource-limits", error.to_string()))?;
    if limits.max_artifacts == 0
        || limits.max_replay_runs == 0
        || limits.max_artifact_bytes == 0
        || limits.max_evaluator_bytes == 0
        || limits.max_stderr_bytes == 0
        || limits.max_path_bytes == 0
        || limits.max_option_bytes == 0
        || limits.max_diagnostic_bytes == 0
        || limits.evaluator_timeout_milliseconds == 0
        || limits.evaluator_poll_milliseconds == 0
    {
        return Err(ShellError::new(
            "resource-limits",
            "every resource limit must be non-zero",
        ));
    }
    Ok(limits)
}

// r[impl nickel_export.core.manifest_integrity_verification]
fn execute_verify(options: &VerifyOptions) -> Result<(), ShellError> {
    let limits = resource_limits()?;
    let root = canonical_root(&options.root)?;
    let manifest_bytes = read_root_path(
        &root,
        &options.manifest,
        "verify-manifest",
        limits.max_artifact_bytes,
    )?;
    let wire: ExportManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| ShellError::new("verify-manifest", error.to_string()))?;
    let manifest = verify_manifest_integrity(wire)
        .map_err(|error| ShellError::new("verify-manifest", error.to_string()))?;
    let artifacts_checked = if options.check_artifacts {
        verify_manifest_artifact_files(&root, &manifest, &limits)?
    } else {
        0
    };
    let report = IntegrityReport {
        schema: "onix-nickel-export-integrity-report/v1",
        manifest_identity: manifest.manifest_identity.clone(),
        artifacts_checked,
        claim: "internal canonical integrity only; freshness and semantic correctness are not proven",
    };
    let rendered = serde_json::to_string(&report)
        .map_err(|error| ShellError::new("verify-report", error.to_string()))?;
    println!("{rendered}");
    Ok(())
}

fn verify_manifest_artifact_files(
    root: &Path,
    manifest: &VerifiedManifest,
    limits: &ResourceLimits,
) -> Result<usize, ShellError> {
    let mut paths = BTreeSet::new();
    for export in &manifest.exports {
        paths.insert(export.source.path.clone());
        paths.extend(
            export
                .dependencies
                .iter()
                .map(|artifact| artifact.path.clone()),
        );
        paths.insert(export.output.path.clone());
    }
    let bytes = paths
        .iter()
        .map(|path| {
            read_root_file(root, path, "verify-artifact", limits.max_artifact_bytes)
                .map(|bytes| (path.clone(), bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let materials = bytes
        .iter()
        .map(|(path, bytes)| ArtifactMaterial {
            path,
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();
    verify_supplied_artifacts(manifest, &materials)
        .map_err(|error| ShellError::new("verify-artifact", error.to_string()))
}

fn parse_verify_args(args: &[String]) -> Result<VerifyOptions, ShellError> {
    let mut root = None;
    let mut manifest = None;
    let mut check_artifacts = false;
    let mut index = FIRST_OPTION_INDEX;
    while index < args.len() {
        match args[index].as_str() {
            FLAG_ROOT => root = Some(PathBuf::from(next_value(args, &mut index, FLAG_ROOT)?)),
            FLAG_MANIFEST => {
                manifest = Some(PathBuf::from(next_value(args, &mut index, FLAG_MANIFEST)?));
            }
            FLAG_CHECK_ARTIFACTS if !check_artifacts => check_artifacts = true,
            FLAG_CHECK_ARTIFACTS => {
                return Err(ShellError::new(
                    "arguments",
                    format!("duplicate `{FLAG_CHECK_ARTIFACTS}`; {VERIFY_USAGE}"),
                ));
            }
            unknown => {
                return Err(ShellError::new(
                    "arguments",
                    format!("unknown argument `{unknown}`; {VERIFY_USAGE}"),
                ));
            }
        }
        index += 1;
    }
    let options = VerifyOptions {
        root: required(root, FLAG_ROOT)?,
        manifest: required(manifest, FLAG_MANIFEST)?,
        check_artifacts,
    };
    validate_relative_shell_path(&options.manifest, FLAG_MANIFEST)?;
    Ok(options)
}

// r[impl nickel_export.shell.authority]
fn execute(options: &CliOptions) -> Result<(), ShellError> {
    let limits = resource_limits()?;
    let loaded = load_export(options, &limits)?;
    let evaluated = evaluate_export(options, &loaded, &limits)?;
    let dependencies = loaded
        .dependency_bytes
        .iter()
        .map(|(path, bytes)| ArtifactMaterial {
            path: path.as_str(),
            bytes: bytes.as_slice(),
        })
        .collect();
    let observation = EvaluationObservation {
        request: &loaded.request,
        source: ArtifactMaterial {
            path: &loaded.request.source,
            bytes: &loaded.source_bytes,
        },
        dependencies,
        output: ArtifactMaterial {
            path: &loaded.request.destination,
            bytes: &evaluated.output_bytes,
        },
        evaluator: &evaluated.evaluator,
        observed_dependencies: Vec::new(),
        diagnostics: Vec::new(),
    };
    let receipt = build_receipt(&observation)
        .map_err(|error| ShellError::new("admit-receipt", error.to_string()))?;
    let manifest = build_manifest(core::slice::from_ref(&receipt))
        .map_err(|error| ShellError::new("build-manifest", error.to_string()))?;
    match options.mode {
        Mode::Write => write_artifacts(
            &loaded.root,
            &loaded.request,
            &evaluated.output_bytes,
            &options.manifest,
            &manifest,
        )?,
        Mode::Check => check_artifacts(
            &loaded.root,
            &loaded.request,
            &evaluated.output_bytes,
            &options.manifest,
            &manifest,
            &limits,
        )?,
    }
    if let Some(replay) = &evaluated.replay {
        let rendered_replay = serde_json::to_string(replay)
            .map_err(|error| ShellError::new("render-replay", error.to_string()))?;
        println!("{rendered_replay}");
    }
    let rendered_receipt = serde_json::to_string(&receipt)
        .map_err(|error| ShellError::new("render-receipt", error.to_string()))?;
    println!("{rendered_receipt}");
    Ok(())
}

fn load_export(options: &CliOptions, limits: &ResourceLimits) -> Result<LoadedExport, ShellError> {
    let root = canonical_root(&options.root)?;
    let request_bytes =
        read_root_path(&root, &options.spec, "read-spec", limits.max_artifact_bytes)?;
    let request: ExportRequest = serde_json::from_slice(&request_bytes).map_err(|error| {
        ShellError::new("parse-spec", format!("{}: {error}", options.spec.display()))
    })?;
    let request = normalize_request(&request)
        .map_err(|error| ShellError::new("validate-spec", error.to_string()))?;
    validate_shell_contract(&request)?;
    let source_bytes = read_root_file(
        &root,
        &request.source,
        "read-source",
        limits.max_artifact_bytes,
    )?;
    let dependency_bytes = request
        .dependencies
        .iter()
        .map(|path| {
            read_root_file(&root, path, "read-dependency", limits.max_artifact_bytes)
                .map(|bytes| (path.clone(), bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LoadedExport {
        root,
        request,
        source_bytes,
        dependency_bytes,
    })
}

fn evaluate_export(
    options: &CliOptions,
    loaded: &LoadedExport,
    limits: &ResourceLimits,
) -> Result<EvaluatedExport, ShellError> {
    let captured = capture_files(
        &loaded.request,
        &loaded.source_bytes,
        &loaded.dependency_bytes,
    )?;
    let snapshot = materialize_snapshot(&loaded.request, &captured)?;
    let evaluator_program = resolve_evaluator_program(&options.evaluator)?;
    let resource_profile_identity = blake3_identity(DEFAULT_RESOURCE_LIMITS_JSON.as_bytes());
    let canonical_plan = canonical_evaluation_plan(&loaded.request, &resource_profile_identity);
    let plan_identity = canonical_plan_identity(&canonical_plan)?;
    let artifact_identity =
        evaluator_artifact_identity(&evaluator_program, limits.max_evaluator_bytes)?;
    let plan = evaluation_plan(&evaluator_program, &canonical_plan, &snapshot.root);
    verify_evaluator_version(&plan.program, &options.evaluator_version)?;
    verify_evaluator_artifact(
        &evaluator_program,
        &artifact_identity,
        limits.max_evaluator_bytes,
    )?;
    let replay_profile = replay_profile(options.replay_runs, limits.max_replay_runs)?;
    let (output_bytes, replay) = if let Some(profile) = replay_profile {
        let assessment = execute_replay_runs(
            profile,
            &plan_identity,
            &artifact_identity,
            &resource_profile_identity,
            || {
                run_bound_evaluator_once(&plan, &evaluator_program, &artifact_identity, limits)
                    .map_err(|error| error.stage)
            },
        )?;
        let (output_bytes, report) = require_replay_agreement(assessment)?;
        (output_bytes, Some(report))
    } else {
        (
            run_bound_evaluator_once(&plan, &evaluator_program, &artifact_identity, limits)?,
            None,
        )
    };
    Ok(EvaluatedExport {
        output_bytes,
        evaluator: EvaluatorDescriptor {
            identity: options.evaluator_identity.clone(),
            artifact_identity,
            closure_identity: String::new(),
            plan_identity,
            version: options.evaluator_version.clone(),
            options: evaluator_options(&canonical_plan),
            import_path_policy: ImportPathPolicy::SnapshotOnly,
        },
        replay,
    })
}

fn run_bound_evaluator_once(
    plan: &EvaluationPlan,
    evaluator_program: &Path,
    artifact_identity: &str,
    limits: &ResourceLimits,
) -> Result<Vec<u8>, ShellError> {
    let result = run_evaluator(plan, limits);
    verify_evaluator_artifact(
        evaluator_program,
        artifact_identity,
        limits.max_evaluator_bytes,
    )?;
    result.map(|output| output.stdout)
}

fn parse_args(args: &[String]) -> Result<CliOptions, ShellError> {
    if args.get(1).map(String::as_str) != Some(COMMAND_EXPORT) {
        return Err(ShellError::new("arguments", USAGE));
    }
    let mut spec = None;
    let mut root = None;
    let mut evaluator = None;
    let mut evaluator_identity = None;
    let mut evaluator_version = None;
    let mut manifest = None;
    let mut replay_runs = None;
    let mut mode = None;
    let mut index = FIRST_OPTION_INDEX;
    while index < args.len() {
        let flag = &args[index];
        match flag.as_str() {
            FLAG_WRITE => set_mode(&mut mode, Mode::Write)?,
            FLAG_CHECK => set_mode(&mut mode, Mode::Check)?,
            FLAG_SPEC => spec = Some(PathBuf::from(next_value(args, &mut index, FLAG_SPEC)?)),
            FLAG_ROOT => root = Some(PathBuf::from(next_value(args, &mut index, FLAG_ROOT)?)),
            FLAG_EVALUATOR => {
                evaluator = Some(PathBuf::from(next_value(args, &mut index, FLAG_EVALUATOR)?));
            }
            FLAG_EVALUATOR_IDENTITY => {
                evaluator_identity =
                    Some(next_value(args, &mut index, FLAG_EVALUATOR_IDENTITY)?.to_string());
            }
            FLAG_EVALUATOR_VERSION => {
                evaluator_version =
                    Some(next_value(args, &mut index, FLAG_EVALUATOR_VERSION)?.to_string());
            }
            FLAG_MANIFEST => {
                manifest = Some(PathBuf::from(next_value(args, &mut index, FLAG_MANIFEST)?));
            }
            FLAG_REPLAY_RUNS if replay_runs.is_none() => {
                replay_runs = Some(parse_replay_runs(next_value(
                    args,
                    &mut index,
                    FLAG_REPLAY_RUNS,
                )?)?);
            }
            FLAG_REPLAY_RUNS => {
                return Err(ShellError::new(
                    "arguments",
                    format!("duplicate `{FLAG_REPLAY_RUNS}`; {USAGE}"),
                ));
            }
            unknown => {
                return Err(ShellError::new(
                    "arguments",
                    format!("unknown argument `{unknown}`; {USAGE}"),
                ));
            }
        }
        index += 1;
    }
    let options = CliOptions {
        spec: required(spec, FLAG_SPEC)?,
        root: required(root, FLAG_ROOT)?,
        evaluator: required(evaluator, FLAG_EVALUATOR)?,
        evaluator_identity: required(evaluator_identity, FLAG_EVALUATOR_IDENTITY)?,
        evaluator_version: required(evaluator_version, FLAG_EVALUATOR_VERSION)?,
        manifest: required(manifest, FLAG_MANIFEST)?,
        replay_runs,
        mode: required(mode, "--write or --check")?,
    };
    require_nonempty(&options.evaluator_identity, FLAG_EVALUATOR_IDENTITY)?;
    require_nonempty(&options.evaluator_version, FLAG_EVALUATOR_VERSION)?;
    validate_relative_shell_path(&options.spec, FLAG_SPEC)?;
    validate_relative_shell_path(&options.manifest, FLAG_MANIFEST)?;
    Ok(options)
}

fn next_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag: &str,
) -> Result<&'a str, ShellError> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| ShellError::new("arguments", format!("missing value for `{flag}`")))
}

fn set_mode(mode: &mut Option<Mode>, value: Mode) -> Result<(), ShellError> {
    if mode.replace(value).is_some() {
        return Err(ShellError::new(
            "arguments",
            "choose exactly one of --write or --check",
        ));
    }
    Ok(())
}

fn required<T>(value: Option<T>, flag: &str) -> Result<T, ShellError> {
    value.ok_or_else(|| {
        ShellError::new(
            "arguments",
            format!("missing required argument `{flag}`; {USAGE}"),
        )
    })
}

fn require_nonempty(value: &str, flag: &str) -> Result<(), ShellError> {
    if value.trim().is_empty() {
        Err(ShellError::new(
            "arguments",
            format!("`{flag}` must not be empty"),
        ))
    } else {
        Ok(())
    }
}

fn parse_replay_runs(value: &str) -> Result<usize, ShellError> {
    let runs = value.parse::<usize>().map_err(|error| {
        ShellError::new(
            "arguments",
            format!("`{FLAG_REPLAY_RUNS}` requires an integer: {error}"),
        )
    })?;
    if runs < REPLAY_MINIMUM_RUNS {
        return Err(ShellError::new(
            "arguments",
            format!("`{FLAG_REPLAY_RUNS}` requires at least {REPLAY_MINIMUM_RUNS} sequential runs"),
        ));
    }
    Ok(runs)
}

fn replay_profile(
    requested_runs: Option<usize>,
    maximum_runs: usize,
) -> Result<Option<ReplayProfile>, ShellError> {
    let Some(requested_runs) = requested_runs else {
        return Ok(None);
    };
    if requested_runs > maximum_runs {
        return Err(ShellError::new(
            "replay-profile",
            format!(
                "requested {requested_runs} replay runs exceeds configured maximum {maximum_runs}"
            ),
        ));
    }
    Ok(Some(ReplayProfile {
        requested_runs,
        maximum_runs,
    }))
}

fn validate_shell_contract(request: &ExportRequest) -> Result<(), ShellError> {
    if request.contract.is_empty() {
        return Ok(());
    }
    validate_relative_shell_path(Path::new(&request.contract), "contract")?;
    if request.dependencies.contains(&request.contract) {
        Ok(())
    } else {
        Err(ShellError::new(
            "validate-spec",
            format!(
                "CLI contract file `{}` must also be declared as an exact dependency",
                request.contract
            ),
        ))
    }
}

// r[impl nickel_export.shell.captured_input_evaluation]
fn capture_files(
    request: &ExportRequest,
    source_bytes: &[u8],
    dependency_bytes: &[(String, Vec<u8>)],
) -> Result<Vec<CapturedFile>, ShellError> {
    if request.dependencies.len() != dependency_bytes.len() {
        return Err(ShellError::new(
            "capture-inputs",
            "captured dependency count differs from normalized request",
        ));
    }
    let mut captured = Vec::with_capacity(request.dependencies.len() + 1);
    captured.push(CapturedFile {
        path: request.source.clone(),
        bytes: source_bytes.to_vec(),
    });
    for (declared, (path, bytes)) in request.dependencies.iter().zip(dependency_bytes) {
        if declared != path {
            return Err(ShellError::new(
                "capture-inputs",
                format!("captured dependency `{path}` differs from declared `{declared}`"),
            ));
        }
        captured.push(CapturedFile {
            path: path.clone(),
            bytes: bytes.clone(),
        });
    }
    Ok(captured)
}

fn materialize_snapshot(
    request: &ExportRequest,
    captured: &[CapturedFile],
) -> Result<EvaluationSnapshot, ShellError> {
    let snapshot = EvaluationSnapshot {
        root: create_snapshot_root()?,
    };
    for file in captured {
        write_snapshot_file(&snapshot.root, file)?;
    }
    for import_path in &request.import_paths {
        fs::create_dir_all(snapshot.root.join(import_path)).map_err(|error| {
            ShellError::new("snapshot-import-path", format!("{import_path}: {error}"))
        })?;
    }
    fs::create_dir_all(snapshot.root.join(SNAPSHOT_PACKAGE_CACHE))
        .map_err(|error| ShellError::new("snapshot-package-cache", error.to_string()))?;
    Ok(snapshot)
}

fn create_snapshot_root() -> Result<PathBuf, ShellError> {
    let parent = std::env::temp_dir();
    let process_id = std::process::id();
    for _ in 0..SNAPSHOT_CREATE_ATTEMPTS {
        let sequence = SNAPSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            "{SNAPSHOT_DIRECTORY_PREFIX}-{process_id}-{sequence}"
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(ShellError::new(
                    "snapshot-create",
                    format!("{}: {error}", candidate.display()),
                ));
            }
        }
    }
    Err(ShellError::new(
        "snapshot-create",
        "exhausted bounded snapshot path attempts",
    ))
}

fn write_snapshot_file(root: &Path, file: &CapturedFile) -> Result<(), ShellError> {
    let destination = root.join(&file.path);
    let Some(parent) = destination.parent() else {
        return Err(ShellError::new(
            "snapshot-write",
            format!("{} has no parent", destination.display()),
        ));
    };
    fs::create_dir_all(parent).map_err(|error| {
        ShellError::new("snapshot-write", format!("{}: {error}", parent.display()))
    })?;
    fs::write(&destination, &file.bytes).map_err(|error| {
        ShellError::new(
            "snapshot-write",
            format!("{}: {error}", destination.display()),
        )
    })
}

fn resolve_evaluator_program(program: &Path) -> Result<PathBuf, ShellError> {
    if program.is_absolute() || program.components().count() > 1 {
        return canonical_evaluator_program(program);
    }
    let path = std::env::var_os("PATH")
        .ok_or_else(|| ShellError::new("evaluator-path", "PATH is unavailable"))?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(program);
        if candidate.is_file() {
            return canonical_evaluator_program(&candidate);
        }
    }
    Err(ShellError::new(
        "evaluator-path",
        format!("could not resolve `{}`", program.display()),
    ))
}

fn canonical_evaluator_program(program: &Path) -> Result<PathBuf, ShellError> {
    let canonical = program.canonicalize().map_err(|error| {
        ShellError::new("evaluator-path", format!("{}: {error}", program.display()))
    })?;
    if canonical.is_file() {
        Ok(canonical)
    } else {
        Err(ShellError::new(
            "evaluator-path",
            format!("{} is not a file", canonical.display()),
        ))
    }
}

// r[impl nickel_export.shell.evaluator_execution_identity]
fn canonical_evaluation_plan(
    request: &ExportRequest,
    resource_profile_identity: &str,
) -> CanonicalEvaluationPlan {
    CanonicalEvaluationPlan {
        source: request.source.clone(),
        import_paths: request.import_paths.clone(),
        selector: request.selector.clone(),
        contract: request.contract.clone(),
        format: evaluator_format(request.format).to_string(),
        color: "never".to_string(),
        package_cache_policy: "private-empty".to_string(),
        resource_profile_identity: resource_profile_identity.to_string(),
    }
}

fn canonical_plan_identity(plan: &CanonicalEvaluationPlan) -> Result<String, ShellError> {
    let bytes = serde_json::to_vec(plan)
        .map_err(|error| ShellError::new("evaluator-plan", error.to_string()))?;
    Ok(blake3_identity(&bytes))
}

fn evaluation_plan(
    program: &Path,
    canonical: &CanonicalEvaluationPlan,
    root: &Path,
) -> EvaluationPlan {
    let mut args = vec![
        OsString::from("export"),
        OsString::from("--color"),
        OsString::from(&canonical.color),
        OsString::from("--package-cache-dir"),
        root.join(SNAPSHOT_PACKAGE_CACHE).into_os_string(),
        OsString::from("--format"),
        OsString::from(&canonical.format),
        OsString::from(&canonical.source),
    ];
    for import_path in &canonical.import_paths {
        args.push(OsString::from("--import-path"));
        args.push(root.join(import_path).into_os_string());
    }
    if !canonical.selector.is_empty() {
        args.push(OsString::from("--field"));
        args.push(OsString::from(&canonical.selector));
    }
    if !canonical.contract.is_empty() {
        args.push(OsString::from("--apply-contract"));
        args.push(root.join(&canonical.contract).into_os_string());
    }
    EvaluationPlan {
        program: program.to_path_buf(),
        args,
        current_dir: root.to_path_buf(),
    }
}

const fn evaluator_format(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Json => "json",
        ExportFormat::Toml => "toml",
        ExportFormat::Yaml => "yaml",
        ExportFormat::Raw => "text",
    }
}

fn evaluator_options(plan: &CanonicalEvaluationPlan) -> Vec<String> {
    let mut options = vec![
        format!("color={}", plan.color),
        format!("format={}", plan.format),
        format!("package-cache={}", plan.package_cache_policy),
        format!("resource-profile={}", plan.resource_profile_identity),
    ];
    options.extend(
        plan.import_paths
            .iter()
            .map(|path| format!("import-path={path}")),
    );
    if !plan.selector.is_empty() {
        options.push(format!("selector={}", plan.selector));
    }
    if !plan.contract.is_empty() {
        options.push(format!("contract={}", plan.contract));
    }
    options
}

// r[impl nickel_export.shell.determinism_replay]
fn execute_replay_runs<F>(
    profile: ReplayProfile,
    plan_identity: &str,
    evaluator_artifact_identity: &str,
    resource_profile_identity: &str,
    mut execute_run: F,
) -> Result<ReplayAssessment, ShellError>
where
    F: FnMut() -> Result<Vec<u8>, &'static str>,
{
    if profile.requested_runs < REPLAY_MINIMUM_RUNS || profile.requested_runs > profile.maximum_runs
    {
        return Err(ShellError::new(
            "replay-profile",
            "replay profile violates its typed run-count bounds",
        ));
    }
    let mut attempts = Vec::with_capacity(profile.requested_runs);
    for _ in 0..profile.requested_runs {
        match execute_run() {
            Ok(bytes) => attempts.push(ReplayAttempt::Success(bytes)),
            Err(stage) => {
                attempts.push(ReplayAttempt::Failure(stage));
                break;
            }
        }
    }
    assess_replay(
        profile,
        plan_identity,
        evaluator_artifact_identity,
        resource_profile_identity,
        &attempts,
    )
}

fn assess_replay(
    profile: ReplayProfile,
    plan_identity: &str,
    evaluator_artifact_identity: &str,
    resource_profile_identity: &str,
    attempts: &[ReplayAttempt],
) -> Result<ReplayAssessment, ShellError> {
    if attempts.is_empty() || attempts.len() > profile.requested_runs {
        return Err(ShellError::new(
            "replay-assessment",
            "replay attempts violate the selected profile",
        ));
    }
    let mut outcomes = Vec::with_capacity(attempts.len());
    let mut reference_output: Option<Vec<u8>> = None;
    let mut saw_failure = false;
    let mut saw_divergence = false;
    for (index, attempt) in attempts.iter().enumerate() {
        match attempt {
            ReplayAttempt::Success(bytes) => {
                if reference_output
                    .as_ref()
                    .is_some_and(|reference| reference != bytes)
                {
                    saw_divergence = true;
                } else if reference_output.is_none() {
                    reference_output = Some(bytes.clone());
                }
                let output_bytes = u64::try_from(bytes.len()).map_err(|_| {
                    ShellError::new("replay-assessment", "output byte length overflowed")
                })?;
                outcomes.push(ReplayRunOutcome {
                    run: index + 1,
                    status: ReplayRunStatus::Success,
                    output_identity: blake3_identity(bytes),
                    output_bytes,
                    failure_stage: String::new(),
                });
            }
            ReplayAttempt::Failure(stage) => {
                saw_failure = true;
                outcomes.push(ReplayRunOutcome {
                    run: index + 1,
                    status: ReplayRunStatus::Failure,
                    output_identity: String::new(),
                    output_bytes: 0,
                    failure_stage: (*stage).to_string(),
                });
            }
        }
    }
    let verdict = if saw_failure || attempts.len() != profile.requested_runs {
        ReplayVerdict::Failure
    } else if saw_divergence {
        ReplayVerdict::Divergence
    } else {
        ReplayVerdict::Agreement
    };
    let mut report = ReplayReport {
        schema: REPLAY_REPORT_SCHEMA,
        profile,
        plan_identity: plan_identity.to_string(),
        evaluator_artifact_identity: evaluator_artifact_identity.to_string(),
        resource_profile_identity: resource_profile_identity.to_string(),
        outcomes,
        verdict,
        report_identity: String::new(),
        non_claim: REPLAY_NON_CLAIM,
    };
    report.report_identity = blake3_identity(&canonical_replay_report_bytes(&report)?);
    let agreed_output = if verdict == ReplayVerdict::Agreement {
        reference_output
    } else {
        None
    };
    Ok(ReplayAssessment {
        report,
        agreed_output,
    })
}

fn require_replay_agreement(
    assessment: ReplayAssessment,
) -> Result<(Vec<u8>, ReplayReport), ShellError> {
    let verdict = assessment.report.verdict;
    let Some(output_bytes) = assessment.agreed_output else {
        let (stage, message) = match verdict {
            ReplayVerdict::Divergence => (
                "replay-divergence",
                "selected replay runs produced different exact output bytes",
            ),
            ReplayVerdict::Failure => (
                "replay-run-failure",
                "a selected replay run failed before agreement",
            ),
            ReplayVerdict::Agreement => (
                "replay-assessment",
                "agreement report did not retain an output",
            ),
        };
        return Err(ShellError::new(stage, message).with_replay(assessment.report));
    };
    Ok((output_bytes, assessment.report))
}

fn canonical_replay_report_bytes(report: &ReplayReport) -> Result<Vec<u8>, ShellError> {
    let mut output = Vec::new();
    append_replay_bytes(&mut output, report.schema.as_bytes())?;
    append_replay_count(&mut output, report.profile.requested_runs)?;
    append_replay_count(&mut output, report.profile.maximum_runs)?;
    append_replay_bytes(&mut output, report.plan_identity.as_bytes())?;
    append_replay_bytes(&mut output, report.evaluator_artifact_identity.as_bytes())?;
    append_replay_bytes(&mut output, report.resource_profile_identity.as_bytes())?;
    append_replay_count(&mut output, report.outcomes.len())?;
    for outcome in &report.outcomes {
        append_replay_count(&mut output, outcome.run)?;
        append_replay_bytes(&mut output, outcome.status.as_str().as_bytes())?;
        append_replay_bytes(&mut output, outcome.output_identity.as_bytes())?;
        append_replay_u64(&mut output, outcome.output_bytes);
        append_replay_bytes(&mut output, outcome.failure_stage.as_bytes())?;
    }
    append_replay_bytes(&mut output, report.verdict.as_str().as_bytes())?;
    append_replay_bytes(&mut output, report.non_claim.as_bytes())?;
    Ok(output)
}

fn append_replay_count(output: &mut Vec<u8>, count: usize) -> Result<(), ShellError> {
    let count = u64::try_from(count)
        .map_err(|_| ShellError::new("replay-canonicalize", "count overflowed u64"))?;
    append_replay_u64(output, count);
    Ok(())
}

fn append_replay_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn append_replay_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), ShellError> {
    append_replay_count(output, bytes.len())?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn evaluator_artifact_identity(program: &Path, max_bytes: u64) -> Result<String, ShellError> {
    let bytes = read_file_bounded(program, "evaluator-artifact", max_bytes)?;
    Ok(blake3_identity(&bytes))
}

fn verify_evaluator_artifact(
    program: &Path,
    expected: &str,
    max_bytes: u64,
) -> Result<(), ShellError> {
    let actual = evaluator_artifact_identity(program, max_bytes)?;
    if actual == expected {
        Ok(())
    } else {
        Err(ShellError::new(
            "evaluator-artifact",
            format!("{} changed during evaluation", program.display()),
        ))
    }
}

fn verify_evaluator_version(program: &Path, expected: &str) -> Result<(), ShellError> {
    let output = evaluator_command(program)
        .arg("--version")
        .output()
        .map_err(|error| ShellError::new("evaluator-version", error.to_string()))?;
    if !output.status.success() {
        return Err(ShellError::new(
            "evaluator-version",
            format!("version command failed with status {}", output.status),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_token = expected
        .strip_prefix(NICKEL_PACKAGE_VERSION_PREFIX)
        .unwrap_or(expected);
    if stdout
        .split_whitespace()
        .any(|token| token == expected_token)
    {
        Ok(())
    } else {
        Err(ShellError::new(
            "evaluator-version",
            format!(
                "evaluator version output `{}` does not contain expected token `{expected_token}`",
                stdout.trim()
            ),
        ))
    }
}

fn evaluator_command(program: &Path) -> Command {
    let mut command = Command::new(program);
    command.env_clear();
    command
}

// r[impl nickel_export.shell.bounded_evaluation]
fn run_evaluator(plan: &EvaluationPlan, limits: &ResourceLimits) -> Result<Output, ShellError> {
    let mut child = evaluator_command(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| ShellError::new("evaluator-spawn", error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ShellError::new("evaluator-stdout", "stdout pipe is unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ShellError::new("evaluator-stderr", "stderr pipe is unavailable"))?;
    let max_stdout = limits.max_artifact_bytes;
    let max_stderr = limits.max_stderr_bytes;
    let stdout_reader =
        std::thread::spawn(move || read_stream_bounded(stdout, max_stdout, "evaluator-stdout"));
    let stderr_reader =
        std::thread::spawn(move || read_stream_bounded(stderr, max_stderr, "evaluator-stderr"));
    let timeout = Duration::from_millis(limits.evaluator_timeout_milliseconds);
    let poll = Duration::from_millis(limits.evaluator_poll_milliseconds);
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| ShellError::new("evaluator-wait", error.to_string()))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_bounded_reader(stdout_reader, "evaluator-stdout");
            let _ = join_bounded_reader(stderr_reader, "evaluator-stderr");
            return Err(ShellError::new(
                "evaluator-timeout",
                "evaluator exceeded the configured deadline",
            ));
        }
        std::thread::sleep(poll);
    };
    let stdout = join_bounded_reader(stdout_reader, "evaluator-stdout")?;
    let stderr = join_bounded_reader(stderr_reader, "evaluator-stderr")?;
    let output = Output {
        status,
        stdout,
        stderr,
    };
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ShellError::new(
            "evaluator-failure",
            format!("status {}: {stderr}", output.status),
        ))
    }
}

fn read_stream_bounded(
    mut reader: impl Read,
    max_bytes: u64,
    stage: &'static str,
) -> Result<Vec<u8>, ShellError> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; STREAM_BUFFER_BYTES];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| ShellError::new(stage, error.to_string()))?;
        if read == 0 {
            return Ok(output);
        }
        let next = output
            .len()
            .checked_add(read)
            .ok_or_else(|| ShellError::new(stage, "stream byte length overflowed"))?;
        let next_u64 = u64::try_from(next)
            .map_err(|_| ShellError::new(stage, "stream byte length overflowed"))?;
        if next_u64 > max_bytes {
            return Err(ShellError::new(
                stage,
                "stream exceeds the configured byte bound",
            ));
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

fn join_bounded_reader(
    handle: std::thread::JoinHandle<Result<Vec<u8>, ShellError>>,
    stage: &'static str,
) -> Result<Vec<u8>, ShellError> {
    handle
        .join()
        .map_err(|_| ShellError::new(stage, "stream reader thread failed"))?
}

/// Atomically publish a pointer to an already complete generation directory.
///
/// Consumers opting into generation-pointer mode read the pointer file as a
/// repository-relative directory path. Plain-file export mode continues to use
/// the fail-closed two-artifact transaction protocol.
///
/// # Errors
///
/// Rejects unsafe paths, missing generation directories, lock contention, and
/// staging or publication failures.
pub fn publish_generation_pointer(
    root: &Path,
    generation: &Path,
    pointer: &Path,
) -> Result<(), ShellError> {
    validate_relative_shell_path(generation, "generation-directory")?;
    validate_relative_shell_path(pointer, "generation-pointer")?;
    let canonical_generation = root.join(generation).canonicalize().map_err(|error| {
        ShellError::new(
            "generation-pointer",
            format!("{}: {error}", generation.display()),
        )
    })?;
    if !canonical_generation.starts_with(root) || !canonical_generation.is_dir() {
        return Err(ShellError::new(
            "generation-pointer",
            "generation must be an existing directory inside the repository root",
        ));
    }
    let _lock = acquire_materialization_lock(root)?;
    reject_incomplete_transaction(root)?;
    let generation_bytes = generation
        .to_str()
        .ok_or_else(|| ShellError::new("generation-pointer", "generation path is not UTF-8"))?
        .as_bytes();
    let staged = stage_artifact(root, pointer, generation_bytes)?;
    publish_transaction_artifact(root, &staged)
}

// r[impl nickel_export.shell.atomic_materialization]
fn write_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &VerifiedManifest,
) -> Result<(), ShellError> {
    let _lock = acquire_materialization_lock(root)?;
    recover_transaction(root)?;
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| ShellError::new("render-manifest", error.to_string()))?;
    let output_artifact = stage_artifact(root, Path::new(&request.destination), output)?;
    let manifest_artifact = match stage_artifact(root, manifest_path, &manifest_bytes) {
        Ok(artifact) => artifact,
        Err(error) => {
            let _ = fs::remove_file(root.join(&output_artifact.temporary));
            return Err(error);
        }
    };
    let transaction = MaterializationTransaction {
        schema: "onix-nickel-export-materialization-transaction/v1".to_string(),
        output: output_artifact,
        manifest: manifest_artifact,
    };
    if let Err(error) = write_transaction_marker(root, &transaction) {
        let _ = fs::remove_file(root.join(&transaction.output.temporary));
        let _ = fs::remove_file(root.join(&transaction.manifest.temporary));
        return Err(error);
    }
    publish_transaction(root, &transaction)?;
    finish_transaction(root)
}

fn check_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &VerifiedManifest,
    limits: &ResourceLimits,
) -> Result<(), ShellError> {
    let _lock = acquire_materialization_lock(root)?;
    reject_incomplete_transaction(root)?;
    let checked_output = read_root_file(
        root,
        &request.destination,
        "check-output",
        limits.max_artifact_bytes,
    )?;
    if checked_output != output {
        return Err(ShellError::new(
            "check-output",
            format!("`{}` is stale", request.destination),
        ));
    }
    let checked_manifest_bytes = read_root_path(
        root,
        manifest_path,
        "check-manifest",
        limits.max_artifact_bytes,
    )?;
    let checked_manifest: ExportManifest = serde_json::from_slice(&checked_manifest_bytes)
        .map_err(|error| ShellError::new("check-manifest", error.to_string()))?;
    let checked_manifest = admit_manifest(checked_manifest)
        .map_err(|error| ShellError::new("check-manifest", error.to_string()))?;
    verify_manifest_fresh(&checked_manifest, manifest)
        .map_err(|error| ShellError::new("check-manifest", error.to_string()))
}

fn canonical_root(path: &Path) -> Result<PathBuf, ShellError> {
    path.canonicalize()
        .map_err(|error| ShellError::new("root", format!("{}: {error}", path.display())))
}

fn read_file_bounded(
    path: &Path,
    stage: &'static str,
    max_bytes: u64,
) -> Result<Vec<u8>, ShellError> {
    let metadata = fs::metadata(path)
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", path.display())))?;
    if metadata.len() > max_bytes {
        return Err(ShellError::new(
            stage,
            format!("{} exceeds the configured byte bound", path.display()),
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", path.display())))?;
    let actual = u64::try_from(bytes.len())
        .map_err(|_| ShellError::new(stage, "file byte length overflowed"))?;
    if actual > max_bytes {
        return Err(ShellError::new(
            stage,
            format!(
                "{} changed beyond the configured byte bound",
                path.display()
            ),
        ));
    }
    Ok(bytes)
}

fn read_root_file(
    root: &Path,
    path: &str,
    stage: &'static str,
    max_bytes: u64,
) -> Result<Vec<u8>, ShellError> {
    read_root_path(root, Path::new(path), stage, max_bytes)
}

fn read_root_path(
    root: &Path,
    path: &Path,
    stage: &'static str,
    max_bytes: u64,
) -> Result<Vec<u8>, ShellError> {
    validate_relative_shell_path(path, stage)?;
    let candidate = root.join(path);
    reject_symlink_components(root, path, stage)?;
    let canonical = candidate
        .canonicalize()
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", candidate.display())))?;
    if !canonical.starts_with(root) {
        return Err(ShellError::new(
            "unsafe-path",
            format!("{} escapes the repository root", candidate.display()),
        ));
    }
    read_file_bounded(&canonical, stage, max_bytes)
}

fn acquire_materialization_lock(root: &Path) -> Result<MaterializationLock, ShellError> {
    let path = root.join(MATERIALIZATION_LOCK_PATH);
    for _ in 0..LOCK_ACQUIRE_ATTEMPTS {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(std::process::id().to_string().as_bytes())
                    .and_then(|()| file.sync_all())
                    .map_err(|error| ShellError::new("materialization-lock", error.to_string()))?;
                sync_directory(root, "materialization-lock")?;
                return Ok(MaterializationLock { path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if lock_owner_is_live(&path) {
                    return Err(ShellError::new(
                        "materialization-lock",
                        "another materialization operation owns the repository lock",
                    ));
                }
                fs::remove_file(&path).map_err(|remove_error| {
                    ShellError::new("materialization-lock", remove_error.to_string())
                })?;
            }
            Err(error) => {
                return Err(ShellError::new(
                    "materialization-lock",
                    format!("{}: {error}", path.display()),
                ));
            }
        }
    }
    Err(ShellError::new(
        "materialization-lock",
        "exhausted bounded lock acquisition attempts",
    ))
}

fn lock_owner_is_live(path: &Path) -> bool {
    let Ok(owner) = fs::read_to_string(path) else {
        return true;
    };
    let Ok(process_id) = owner.trim().parse::<u32>() else {
        return true;
    };
    process_is_live(process_id)
}

#[cfg(target_os = "linux")]
fn process_is_live(process_id: u32) -> bool {
    Path::new("/proc").join(process_id.to_string()).exists()
}

#[cfg(not(target_os = "linux"))]
const fn process_is_live(_process_id: u32) -> bool {
    true
}

fn stage_artifact(
    root: &Path,
    destination: &Path,
    bytes: &[u8],
) -> Result<TransactionArtifact, ShellError> {
    validate_relative_shell_path(destination, "stage-artifact")?;
    reject_symlink_components(root, destination, "stage-artifact")?;
    let parent = destination.parent().unwrap_or_else(|| Path::new(""));
    let canonical_parent = prepare_destination_parent(root, parent, "stage-artifact")?;
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ShellError::new("stage-artifact", "destination filename is not UTF-8"))?;
    let sequence = MATERIALIZATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staged_name = format!(
        ".{file_name}.{STAGED_FILE_TAG}-{}-{sequence}",
        std::process::id()
    );
    let temporary = parent.join(staged_name);
    let temporary_string = temporary
        .to_str()
        .ok_or_else(|| ShellError::new("stage-artifact", "temporary path is not UTF-8"))?
        .to_string();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(&temporary))
        .map_err(|error| ShellError::new("stage-artifact", error.to_string()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| ShellError::new("stage-artifact", error.to_string()))?;
    sync_directory(&canonical_parent, "stage-artifact")?;
    Ok(TransactionArtifact {
        temporary: temporary_string,
        destination: destination
            .to_str()
            .ok_or_else(|| ShellError::new("stage-artifact", "destination is not UTF-8"))?
            .to_string(),
        identity: blake3_identity(bytes),
    })
}

fn prepare_destination_parent(
    root: &Path,
    relative_parent: &Path,
    stage: &'static str,
) -> Result<PathBuf, ShellError> {
    let parent = root.join(relative_parent);
    fs::create_dir_all(&parent)
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
    let canonical = parent
        .canonicalize()
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
    if !canonical.starts_with(root) {
        return Err(ShellError::new(
            "unsafe-path",
            format!("{} escapes the repository root", parent.display()),
        ));
    }
    Ok(canonical)
}

fn write_transaction_marker(
    root: &Path,
    transaction: &MaterializationTransaction,
) -> Result<(), ShellError> {
    let marker = root.join(TRANSACTION_MARKER_PATH);
    let bytes = serde_json::to_vec_pretty(transaction)
        .map_err(|error| ShellError::new("transaction-marker", error.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .map_err(|error| ShellError::new("transaction-marker", error.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| ShellError::new("transaction-marker", error.to_string()))?;
    sync_directory(root, "transaction-marker")
}

fn recover_transaction(root: &Path) -> Result<(), ShellError> {
    let marker = root.join(TRANSACTION_MARKER_PATH);
    if !marker.exists() {
        return Ok(());
    }
    let bytes = read_file_bounded(
        &marker,
        "transaction-recovery",
        ResourceLimits::DEFAULT.max_artifact_bytes,
    )?;
    let transaction: MaterializationTransaction = serde_json::from_slice(&bytes)
        .map_err(|error| ShellError::new("transaction-recovery", error.to_string()))?;
    if transaction.schema != "onix-nickel-export-materialization-transaction/v1" {
        return Err(ShellError::new(
            "transaction-recovery",
            "unsupported transaction schema",
        ));
    }
    publish_transaction(root, &transaction)?;
    finish_transaction(root)
}

fn publish_transaction(
    root: &Path,
    transaction: &MaterializationTransaction,
) -> Result<(), ShellError> {
    publish_transaction_artifact(root, &transaction.output)?;
    publish_transaction_artifact(root, &transaction.manifest)
}

fn publish_transaction_artifact(
    root: &Path,
    artifact: &TransactionArtifact,
) -> Result<(), ShellError> {
    let temporary = Path::new(&artifact.temporary);
    let destination = Path::new(&artifact.destination);
    validate_relative_shell_path(temporary, "transaction-publish")?;
    validate_relative_shell_path(destination, "transaction-publish")?;
    reject_symlink_components(root, temporary, "transaction-publish")?;
    reject_symlink_components(root, destination, "transaction-publish")?;
    let temporary_path = root.join(temporary);
    let destination_path = root.join(destination);
    let temporary_identity = optional_file_identity(&temporary_path)?;
    let destination_identity = optional_file_identity(&destination_path)?;
    let action = recovery_action(
        temporary_identity.as_deref(),
        destination_identity.as_deref(),
        &artifact.identity,
    )?;
    if action == RecoveryAction::PublishTemporary {
        fs::rename(&temporary_path, &destination_path).map_err(|error| {
            ShellError::new(
                "transaction-publish",
                format!("{}: {error}", destination_path.display()),
            )
        })?;
    }
    let parent = destination_path.parent().unwrap_or(root);
    sync_directory(parent, "transaction-publish")
}

fn optional_file_identity(path: &Path) -> Result<Option<String>, ShellError> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = read_file_bounded(
        path,
        "transaction-publish",
        ResourceLimits::DEFAULT.max_artifact_bytes,
    )?;
    Ok(Some(blake3_identity(&bytes)))
}

fn recovery_action(
    temporary_identity: Option<&str>,
    destination_identity: Option<&str>,
    expected_identity: &str,
) -> Result<RecoveryAction, ShellError> {
    match (temporary_identity, destination_identity) {
        (Some(temporary), _) if temporary == expected_identity => {
            Ok(RecoveryAction::PublishTemporary)
        }
        (None, Some(destination)) if destination == expected_identity => {
            Ok(RecoveryAction::DestinationAlreadyPublished)
        }
        _ => Err(ShellError::new(
            "transaction-publish",
            "neither staged nor destination bytes match the recorded identity",
        )),
    }
}

fn finish_transaction(root: &Path) -> Result<(), ShellError> {
    let marker = root.join(TRANSACTION_MARKER_PATH);
    fs::remove_file(&marker)
        .map_err(|error| ShellError::new("transaction-finish", error.to_string()))?;
    sync_directory(root, "transaction-finish")
}

fn reject_incomplete_transaction(root: &Path) -> Result<(), ShellError> {
    if root.join(TRANSACTION_MARKER_PATH).exists() {
        Err(ShellError::new(
            "transaction-incomplete",
            "an interrupted materialization transaction requires recovery",
        ))
    } else {
        Ok(())
    }
}

fn sync_directory(path: &Path, stage: &'static str) -> Result<(), ShellError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", path.display())))
}

fn reject_symlink_components(
    root: &Path,
    path: &Path,
    stage: &'static str,
) -> Result<(), ShellError> {
    let mut candidate = root.to_path_buf();
    for component in path.components() {
        candidate.push(component);
        match fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ShellError::new(
                    "unsafe-path",
                    format!("{stage} rejects symlink component {}", candidate.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(ShellError::new(
                    stage,
                    format!("inspecting {}: {error}", candidate.display()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_relative_shell_path(path: &Path, subject: &str) -> Result<(), ShellError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(ShellError::new(
            "unsafe-path",
            format!("`{subject}` must be repository-root-relative"),
        ));
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(ShellError::new(
                "unsafe-path",
                format!("`{subject}` must not escape the root"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    const TEST_REPLAY_RUNS: usize = 3;

    fn replay_profile_for_test() -> ReplayProfile {
        ReplayProfile {
            requested_runs: TEST_REPLAY_RUNS,
            maximum_runs: ResourceLimits::DEFAULT.max_replay_runs,
        }
    }

    fn valid_args() -> Vec<String> {
        [
            "nickel-export",
            COMMAND_EXPORT,
            FLAG_SPEC,
            "config/export.json",
            FLAG_ROOT,
            ".",
            FLAG_EVALUATOR,
            "nickel",
            FLAG_EVALUATOR_IDENTITY,
            "nix:nickel",
            FLAG_EVALUATOR_VERSION,
            "nickel-1.13.0",
            FLAG_MANIFEST,
            "generated/manifest.json",
            FLAG_CHECK,
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    #[test]
    fn parser_accepts_complete_check_request() {
        let parsed = parse_args(&valid_args());
        assert!(parsed.is_ok());
        assert!(matches!(
            parsed.map(|options| options.mode),
            Ok(Mode::Check)
        ));
    }

    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn parser_accepts_bounded_replay_and_rejects_invalid_counts() {
        let mut replay = valid_args();
        replay.push(FLAG_REPLAY_RUNS.to_string());
        replay.push(TEST_REPLAY_RUNS.to_string());
        let parsed = parse_args(&replay).unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(parsed.replay_runs, Some(TEST_REPLAY_RUNS));
        assert!(
            replay_profile(parsed.replay_runs, ResourceLimits::DEFAULT.max_replay_runs).is_ok()
        );

        let mut duplicate = replay;
        duplicate.push(FLAG_REPLAY_RUNS.to_string());
        duplicate.push(TEST_REPLAY_RUNS.to_string());
        assert!(parse_args(&duplicate).is_err());

        let mut too_few = valid_args();
        too_few.push(FLAG_REPLAY_RUNS.to_string());
        too_few.push((REPLAY_MINIMUM_RUNS - 1).to_string());
        assert!(parse_args(&too_few).is_err());

        let over_limit = ResourceLimits::DEFAULT.max_replay_runs + 1;
        assert!(replay_profile(Some(over_limit), ResourceLimits::DEFAULT.max_replay_runs).is_err());
    }

    // r[verify nickel_export.core.manifest_integrity_verification]
    #[test]
    fn verify_parser_accepts_read_only_artifact_checks_and_rejects_mutation_flags() {
        let args = [
            "nickel-export",
            COMMAND_VERIFY,
            FLAG_MANIFEST,
            "generated/manifest.json",
            FLAG_ROOT,
            ".",
            FLAG_CHECK_ARTIFACTS,
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let parsed = parse_verify_args(&args).unwrap_or_else(|error| panic_for_test(&error));
        assert!(parsed.check_artifacts);
        assert_eq!(parsed.manifest, PathBuf::from("generated/manifest.json"));

        let mut mutation = args;
        mutation.push(FLAG_WRITE.to_string());
        assert!(parse_verify_args(&mutation).is_err());
    }

    #[test]
    fn parser_rejects_missing_mode_unknown_flags_and_unsafe_paths() {
        let mut missing_mode = valid_args();
        missing_mode.pop();
        assert!(parse_args(&missing_mode).is_err());

        let mut unknown = valid_args();
        unknown.push("--ambient-authority".to_string());
        assert!(parse_args(&unknown).is_err());

        let mut unsafe_spec = valid_args();
        let spec_index = unsafe_spec
            .iter()
            .position(|value| value == FLAG_SPEC)
            .map(|index| index + 1)
            .unwrap_or_default();
        unsafe_spec[spec_index] = "../secret.ncl".to_string();
        assert!(parse_args(&unsafe_spec).is_err());
    }

    #[test]
    fn cli_contract_files_must_be_safe_declared_dependencies() {
        let mut request = ExportRequest {
            schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
            family_id: "tests.config".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: Vec::new(),
            import_paths: Vec::new(),
            selector: String::new(),
            contract: "DynamicEvidenceProfiles".to_string(),
            format: ExportFormat::Json,
            destination: "generated/config.json".to_string(),
            allow_secret_material: false,
        };
        assert!(validate_shell_contract(&request).is_err());
        request.contract = "config/contract.ncl".to_string();
        request.dependencies.push(request.contract.clone());
        assert!(validate_shell_contract(&request).is_ok());
        request.contract = "../contract.ncl".to_string();
        assert!(validate_shell_contract(&request).is_err());
    }

    #[test]
    fn plan_keeps_external_evaluation_in_the_shell() {
        let options = parse_args(&valid_args()).unwrap_or_else(|error| panic_for_test(&error));
        let request = ExportRequest {
            schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
            family_id: "tests.config".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: Vec::new(),
            import_paths: vec!["config/imports".to_string()],
            selector: "value".to_string(),
            contract: "config/contract.ncl".to_string(),
            format: ExportFormat::Raw,
            destination: "generated/value.txt".to_string(),
            allow_secret_material: false,
        };
        let resource_profile_identity = blake3_identity(b"test-resource-profile");
        let canonical = canonical_evaluation_plan(&request, &resource_profile_identity);
        let plan = evaluation_plan(&options.evaluator, &canonical, Path::new("/tmp/root"));
        assert_eq!(plan.program, PathBuf::from("nickel"));
        assert!(plan.args.contains(&OsString::from("text")));
        assert!(plan.args.contains(&OsString::from("--apply-contract")));
        assert!(plan.args.contains(&OsString::from("--package-cache-dir")));
        assert!(plan.args.contains(&OsString::from("never")));
        let identity =
            canonical_plan_identity(&canonical).unwrap_or_else(|error| panic_for_test(&error));
        assert!(identity.starts_with("b3:"));
        assert!(evaluator_options(&canonical).contains(&"format=text".to_string()));
    }

    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn replay_agreement_is_deterministic_and_retains_exact_output() {
        let stable_output = b"stable replay output\n".to_vec();
        let first = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || Ok(stable_output.clone()),
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        let second = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || Ok(stable_output.clone()),
        )
        .unwrap_or_else(|error| panic_for_test(&error));

        assert_eq!(first, second);
        assert_eq!(first.report.verdict, ReplayVerdict::Agreement);
        assert_eq!(first.report.outcomes.len(), TEST_REPLAY_RUNS);
        assert_eq!(first.agreed_output, Some(stable_output));
        let canonical = canonical_replay_report_bytes(&first.report)
            .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(first.report.report_identity, blake3_identity(&canonical));
        let rendered = serde_json::to_string(&first.report).unwrap_or_else(|error| {
            panic_for_test::<String>(&ShellError::new("test", error.to_string()))
        });
        assert!(!rendered.contains("/tmp"));
        assert!(!rendered.contains("process_id"));
        assert!(!rendered.contains("clock"));
    }

    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn replay_divergence_and_failures_withhold_agreed_output() {
        const DIVERGENT_RUN: usize = 2;
        const FAILED_RUN: usize = 2;
        let mut divergent_run = 0;
        let divergent = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || {
                divergent_run += 1;
                if divergent_run == DIVERGENT_RUN {
                    Ok(b"different".to_vec())
                } else {
                    Ok(b"same".to_vec())
                }
            },
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(divergent.report.verdict, ReplayVerdict::Divergence);
        assert!(divergent.agreed_output.is_none());
        assert!(require_replay_agreement(divergent.clone()).is_err());
        assert_ne!(
            divergent.report.outcomes[0].output_identity,
            divergent.report.outcomes[1].output_identity
        );

        for failure_stage in ["evaluator-failure", "evaluator-timeout", "evaluator-stdout"] {
            let mut run = 0;
            let failed = execute_replay_runs(
                replay_profile_for_test(),
                "b3:plan",
                "b3:evaluator",
                "b3:resources",
                || {
                    run += 1;
                    if run == FAILED_RUN {
                        Err(failure_stage)
                    } else {
                        Ok(b"same".to_vec())
                    }
                },
            )
            .unwrap_or_else(|error| panic_for_test(&error));
            assert_eq!(failed.report.verdict, ReplayVerdict::Failure);
            assert!(failed.agreed_output.is_none());
            assert_eq!(failed.report.outcomes.len(), FAILED_RUN);
            assert_eq!(
                failed.report.outcomes[FAILED_RUN - 1].failure_stage,
                failure_stage
            );
        }
    }

    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn sequential_external_evaluator_divergence_and_failure_are_recorded() {
        const TEST_FAILURE_EXIT_CODE: i32 = 7;
        let root = std::env::temp_dir().join(format!(
            "nickel-export-replay-evaluator-test-{}",
            std::process::id()
        ));
        if let Err(error) = fs::remove_dir_all(&root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
            }
        }
        fs::create_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let shell_program = resolve_evaluator_program(Path::new("/bin/sh"))
            .unwrap_or_else(|error| panic_for_test(&error));
        let alternating = EvaluationPlan {
            program: shell_program.clone(),
            args: vec![
                OsString::from("-c"),
                OsString::from(
                    "if test -e replay-state; then printf second; else : > replay-state; printf first; fi",
                ),
            ],
            current_dir: root.clone(),
        };
        let limits = resource_limits().unwrap_or_else(|error| panic_for_test(&error));
        let divergent = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || {
                run_evaluator(&alternating, &limits)
                    .map(|output| output.stdout)
                    .map_err(|error| error.stage)
            },
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(divergent.report.verdict, ReplayVerdict::Divergence);
        assert!(divergent.agreed_output.is_none());

        let failing = EvaluationPlan {
            program: shell_program,
            args: vec![
                OsString::from("-c"),
                OsString::from(format!("exit {TEST_FAILURE_EXIT_CODE}")),
            ],
            current_dir: root.clone(),
        };
        let failed = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || {
                run_evaluator(&failing, &limits)
                    .map(|output| output.stdout)
                    .map_err(|error| error.stage)
            },
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(failed.report.verdict, ReplayVerdict::Failure);
        assert_eq!(failed.report.outcomes[0].failure_stage, "evaluator-failure");
        assert!(failed.agreed_output.is_none());
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    // r[verify nickel_export.shell.captured_input_evaluation]
    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn snapshot_uses_captured_bytes_and_cleans_up() {
        let repository =
            std::env::temp_dir().join(format!("nickel-export-capture-test-{}", std::process::id()));
        if let Err(error) = fs::remove_dir_all(&repository) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
            }
        }
        fs::create_dir_all(repository.join("config")).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let source_path = repository.join("config/source.ncl");
        let captured_bytes = b"{ value = \"captured\" }\n";
        let changed_bytes = b"{ value = \"changed\" }\n";
        fs::write(&source_path, captured_bytes).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let request = ExportRequest {
            schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
            family_id: "tests.snapshot".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: Vec::new(),
            import_paths: vec!["config".to_string()],
            selector: String::new(),
            contract: String::new(),
            format: ExportFormat::Json,
            destination: "generated/config.json".to_string(),
            allow_secret_material: false,
        };
        let source = fs::read(&source_path).unwrap_or_else(|error| {
            panic_for_test::<Vec<u8>>(&ShellError::new("test-read", error.to_string()))
        });
        let captured =
            capture_files(&request, &source, &[]).unwrap_or_else(|error| panic_for_test(&error));
        let snapshot = materialize_snapshot(&request, &captured)
            .unwrap_or_else(|error| panic_for_test(&error));
        let snapshot_root = snapshot.root.clone();
        fs::write(&source_path, changed_bytes).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-mutate", error.to_string()));
        });
        let evaluated_bytes =
            fs::read(snapshot.root.join(&request.source)).unwrap_or_else(|error| {
                panic_for_test::<Vec<u8>>(&ShellError::new("test-read", error.to_string()))
            });

        assert_eq!(evaluated_bytes, captured_bytes);
        assert_ne!(evaluated_bytes, changed_bytes);
        assert!(snapshot.root.join(SNAPSHOT_PACKAGE_CACHE).is_dir());
        drop(snapshot);
        assert!(!snapshot_root.exists());
        fs::remove_dir_all(&repository).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    #[test]
    fn evaluator_command_has_no_ambient_environment() {
        let command = evaluator_command(Path::new("/bin/true"));
        assert_eq!(command.get_envs().count(), 0);
    }

    // r[verify nickel_export.shell.evaluator_execution_identity]
    #[test]
    fn evaluator_artifact_change_fails_closed() {
        let artifact = std::env::temp_dir().join(format!(
            "nickel-export-evaluator-test-{}",
            std::process::id()
        ));
        let original = b"first evaluator artifact";
        let changed = b"changed evaluator artifact";
        fs::write(&artifact, original).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let max_bytes = ResourceLimits::DEFAULT.max_evaluator_bytes;
        let identity = evaluator_artifact_identity(&artifact, max_bytes)
            .unwrap_or_else(|error| panic_for_test(&error));
        assert!(verify_evaluator_artifact(&artifact, &identity, max_bytes).is_ok());
        fs::write(&artifact, changed).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-mutate", error.to_string()));
        });
        assert!(verify_evaluator_artifact(&artifact, &identity, max_bytes).is_err());
        fs::remove_file(&artifact).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    // r[verify nickel_export.shell.bounded_evaluation]
    #[test]
    fn resource_profile_and_stream_bounds_fail_closed() {
        const TEST_STREAM_BOUND: u64 = 4;
        const TEST_TIMEOUT_MILLISECONDS: u64 = 20;
        const TEST_POLL_MILLISECONDS: u64 = 1;

        let limits = resource_limits().unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(limits, ResourceLimits::DEFAULT);
        let exact = read_stream_bounded(
            std::io::Cursor::new(b"four"),
            TEST_STREAM_BOUND,
            "test-stream",
        );
        let oversized = read_stream_bounded(
            std::io::Cursor::new(b"oversized"),
            TEST_STREAM_BOUND,
            "test-stream",
        );
        assert!(exact.is_ok());
        assert!(oversized.is_err());

        let shell_program = resolve_evaluator_program(Path::new("/bin/sh"))
            .unwrap_or_else(|error| panic_for_test(&error));
        let plan = EvaluationPlan {
            program: shell_program,
            args: vec![OsString::from("-c"), OsString::from("while :; do :; done")],
            current_dir: std::env::temp_dir(),
        };
        let timeout_limits = ResourceLimits {
            evaluator_timeout_milliseconds: TEST_TIMEOUT_MILLISECONDS,
            evaluator_poll_milliseconds: TEST_POLL_MILLISECONDS,
            ..limits
        };
        let timeout = run_evaluator(&plan, &timeout_limits);
        assert!(timeout.is_err());
        assert_eq!(
            timeout.err().map(|error| error.stage),
            Some("evaluator-timeout")
        );

        let replay_timeout = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || {
                run_evaluator(&plan, &timeout_limits)
                    .map(|output| output.stdout)
                    .map_err(|error| error.stage)
            },
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(replay_timeout.report.verdict, ReplayVerdict::Failure);
        assert_eq!(
            replay_timeout.report.outcomes[0].failure_stage,
            "evaluator-timeout"
        );

        let oversized_plan = EvaluationPlan {
            program: plan.program,
            args: vec![OsString::from("-c"), OsString::from("printf oversized")],
            current_dir: plan.current_dir,
        };
        let oversized_limits = ResourceLimits {
            max_artifact_bytes: TEST_STREAM_BOUND,
            ..limits
        };
        let replay_oversized = execute_replay_runs(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            || {
                run_evaluator(&oversized_plan, &oversized_limits)
                    .map(|output| output.stdout)
                    .map_err(|error| error.stage)
            },
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(replay_oversized.report.verdict, ReplayVerdict::Failure);
        assert_eq!(
            replay_oversized.report.outcomes[0].failure_stage,
            "evaluator-stdout"
        );
    }

    #[test]
    fn parser_rejects_conflicting_side_effect_modes() {
        let mut args = valid_args();
        args.push(FLAG_WRITE.to_string());
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn failures_serialize_without_a_success_receipt() {
        let error = parse_args(&["nickel-export".to_string()]).err();
        assert!(error.is_some());
        let rendered = error
            .as_ref()
            .and_then(|value| serde_json::to_string(value).ok())
            .unwrap_or_default();
        assert!(rendered.contains(SHELL_ERROR_SCHEMA));
        assert!(rendered.contains("arguments"));
        assert!(!rendered.contains("receipt"));
    }

    // r[verify nickel_export.shell.determinism_replay]
    #[test]
    fn replay_failure_serializes_evidence_without_a_success_receipt() {
        let attempts = [
            ReplayAttempt::Success(b"first".to_vec()),
            ReplayAttempt::Failure("evaluator-timeout"),
        ];
        let assessment = assess_replay(
            replay_profile_for_test(),
            "b3:plan",
            "b3:evaluator",
            "b3:resources",
            &attempts,
        )
        .unwrap_or_else(|error| panic_for_test(&error));
        let error = require_replay_agreement(assessment)
            .err()
            .unwrap_or_else(|| panic_for_test(&ShellError::new("test", "expected replay failure")));
        let rendered = serde_json::to_string(&error).unwrap_or_else(|render_error| {
            panic_for_test::<String>(&ShellError::new("test", render_error.to_string()))
        });
        assert!(rendered.contains(REPLAY_REPORT_SCHEMA));
        assert!(rendered.contains("evaluator-timeout"));
        assert!(!rendered.contains("receipt"));
    }

    // r[verify nickel_export.shell.atomic_materialization]
    #[test]
    fn materialization_lock_and_recovery_are_fail_closed_and_idempotent() {
        let root = std::env::temp_dir().join(format!(
            "nickel-export-transaction-test-{}",
            std::process::id()
        ));
        if let Err(error) = fs::remove_dir_all(&root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
            }
        }
        fs::create_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let lock =
            acquire_materialization_lock(&root).unwrap_or_else(|error| panic_for_test(&error));
        assert!(acquire_materialization_lock(&root).is_err());
        drop(lock);
        let recovered_lock =
            acquire_materialization_lock(&root).unwrap_or_else(|error| panic_for_test(&error));

        let output_bytes = b"transaction output\n";
        let manifest_bytes = b"transaction manifest\n";
        let output_identity = blake3_identity(output_bytes);
        assert_eq!(
            recovery_action(Some(&output_identity), None, &output_identity).ok(),
            Some(RecoveryAction::PublishTemporary)
        );
        assert_eq!(
            recovery_action(None, Some(&output_identity), &output_identity).ok(),
            Some(RecoveryAction::DestinationAlreadyPublished)
        );
        assert!(recovery_action(None, None, &output_identity).is_err());
        let output = stage_artifact(&root, Path::new("generated/output.json"), output_bytes)
            .unwrap_or_else(|error| panic_for_test(&error));
        let manifest = stage_artifact(&root, Path::new("generated/manifest.json"), manifest_bytes)
            .unwrap_or_else(|error| panic_for_test(&error));
        let transaction = MaterializationTransaction {
            schema: "onix-nickel-export-materialization-transaction/v1".to_string(),
            output,
            manifest,
        };
        write_transaction_marker(&root, &transaction)
            .unwrap_or_else(|error| panic_for_test(&error));
        publish_transaction_artifact(&root, &transaction.output)
            .unwrap_or_else(|error| panic_for_test(&error));
        assert!(reject_incomplete_transaction(&root).is_err());
        recover_transaction(&root).unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(
            fs::read(root.join("generated/output.json")).unwrap_or_default(),
            output_bytes
        );
        assert_eq!(
            fs::read(root.join("generated/manifest.json")).unwrap_or_default(),
            manifest_bytes
        );
        assert!(!root.join(TRANSACTION_MARKER_PATH).exists());
        assert!(recover_transaction(&root).is_ok());
        drop(recovered_lock);

        let generation = Path::new("generations/complete");
        fs::create_dir_all(root.join(generation)).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        let pointer = Path::new("generated/current-generation");
        publish_generation_pointer(&root, generation, pointer)
            .unwrap_or_else(|error| panic_for_test(&error));
        assert_eq!(
            fs::read(root.join(pointer)).unwrap_or_default(),
            generation.as_os_str().as_encoded_bytes()
        );
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    #[cfg(unix)]
    #[test]
    fn confined_path_check_accepts_direct_files_and_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root =
            std::env::temp_dir().join(format!("nickel-export-path-test-{}", std::process::id()));
        if let Err(error) = fs::remove_dir_all(&root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
            }
        }
        fs::create_dir_all(root.join("safe")).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        fs::write(root.join("safe/input.ncl"), b"{}\n").unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        symlink(root.join("safe/input.ncl"), root.join("linked.ncl")).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });

        assert!(reject_symlink_components(&root, Path::new("safe/input.ncl"), "test").is_ok());
        assert!(reject_symlink_components(&root, Path::new("linked.ncl"), "test").is_err());
        assert!(reject_symlink_components(&root, Path::new("new/output.json"), "test").is_ok());
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    fn panic_for_test<T>(error: &ShellError) -> T {
        panic!("unexpected shell error: {error}")
    }
}
