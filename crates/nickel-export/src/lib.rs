//! Thin std shell for deterministic Nickel exports.

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;

use nickel_export_core::{
    ArtifactMaterial, EvaluationObservation, EvaluatorDescriptor, ExportFormat, ExportManifest,
    ExportRequest, ImportPathPolicy, ResourceLimits, VerifiedManifest, admit_manifest,
    blake3_identity, build_manifest, build_receipt, normalize_request, verify_manifest_fresh,
};

/// Stable non-zero process exit used for all fail-closed shell errors.
pub const FAILURE_EXIT_CODE: i32 = 2;
/// Structured shell failure schema.
pub const SHELL_ERROR_SCHEMA: &str = "onix-nickel-export-shell-error/v1";
const USAGE: &str = "usage: nickel-export export --spec <request.json> --root <dir> --evaluator <program> --evaluator-identity <identity> --evaluator-version <version> --manifest <relative-path> (--write|--check)";
const COMMAND_EXPORT: &str = "export";
const FIRST_OPTION_INDEX: usize = 2;
const FLAG_SPEC: &str = "--spec";
const FLAG_ROOT: &str = "--root";
const FLAG_EVALUATOR: &str = "--evaluator";
const FLAG_EVALUATOR_IDENTITY: &str = "--evaluator-identity";
const FLAG_EVALUATOR_VERSION: &str = "--evaluator-version";
const FLAG_MANIFEST: &str = "--manifest";
const FLAG_WRITE: &str = "--write";
const FLAG_CHECK: &str = "--check";
const NICKEL_PACKAGE_VERSION_PREFIX: &str = "nickel-lang-cli-";
const SNAPSHOT_DIRECTORY_PREFIX: &str = "nickel-export-snapshot";
const SNAPSHOT_PACKAGE_CACHE: &str = ".nickel-package-cache";
const SNAPSHOT_CREATE_ATTEMPTS: u64 = 64;
const STREAM_BUFFER_BYTES: usize = 8_192;
const DEFAULT_RESOURCE_LIMITS_JSON: &str =
    include_str!("../../../config/generated/resource-limits.json");
static SNAPSHOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Shell error with a stable stage classification.
#[derive(Debug, Serialize)]
pub struct ShellError {
    schema: &'static str,
    stage: &'static str,
    message: String,
}

impl ShellError {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            schema: SHELL_ERROR_SCHEMA,
            stage,
            message: message.into(),
        }
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
    mode: Mode,
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
    output: Output,
    evaluator: EvaluatorDescriptor,
}

/// Execute the CLI shell around the pure core.
///
/// # Errors
///
/// Returns a staged error for malformed arguments, unsafe paths, unreadable
/// inputs, evaluator failure, receipt rejection, stale checked-in artifacts,
/// serialization failure, or output-write failure.
pub fn run(args: &[String]) -> Result<(), ShellError> {
    let options = parse_args(args)?;
    execute(&options)
}

fn resource_limits() -> Result<ResourceLimits, ShellError> {
    let limits: ResourceLimits = serde_json::from_str(DEFAULT_RESOURCE_LIMITS_JSON)
        .map_err(|error| ShellError::new("resource-limits", error.to_string()))?;
    if limits.max_artifacts == 0
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
            bytes: &evaluated.output.stdout,
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
            &evaluated.output.stdout,
            &options.manifest,
            &manifest,
        )?,
        Mode::Check => check_artifacts(
            &loaded.root,
            &loaded.request,
            &evaluated.output.stdout,
            &options.manifest,
            &manifest,
            &limits,
        )?,
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
    let output = run_evaluator(&plan, limits)?;
    verify_evaluator_artifact(
        &evaluator_program,
        &artifact_identity,
        limits.max_evaluator_bytes,
    )?;
    Ok(EvaluatedExport {
        output,
        evaluator: EvaluatorDescriptor {
            identity: options.evaluator_identity.clone(),
            artifact_identity,
            closure_identity: String::new(),
            plan_identity,
            version: options.evaluator_version.clone(),
            options: evaluator_options(&canonical_plan),
            import_path_policy: ImportPathPolicy::SnapshotOnly,
        },
    })
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

fn write_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &VerifiedManifest,
) -> Result<(), ShellError> {
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| ShellError::new("render-manifest", error.to_string()))?;
    write_root_file(
        root,
        Path::new(&request.destination),
        output,
        "write-output",
    )?;
    write_root_file(root, manifest_path, &manifest_bytes, "write-manifest")
}

fn check_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &VerifiedManifest,
    limits: &ResourceLimits,
) -> Result<(), ShellError> {
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

fn write_root_file(
    root: &Path,
    path: &Path,
    bytes: &[u8],
    stage: &'static str,
) -> Result<(), ShellError> {
    validate_relative_shell_path(path, stage)?;
    reject_symlink_components(root, path, stage)?;
    let destination = root.join(path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
        if !canonical_parent.starts_with(root) {
            return Err(ShellError::new(
                "unsafe-path",
                format!("{} escapes the repository root", parent.display()),
            ));
        }
    }
    fs::write(&destination, bytes)
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", destination.display())))
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

    // r[verify nickel_export.shell.captured_input_evaluation]
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
