let MebibyteBytes = 1048576 in
let ArtifactMebibytes = 16 in
let EvaluatorMebibytes = 128 in
let EvaluatorTimeoutMilliseconds = 30000 in
let EvaluatorPollMilliseconds = 10 in
let MaximumArtifacts = 4096 in
let MaximumReplayRuns = 4 in
let MaximumPathBytes = 4096 in
let MaximumOptionBytes = 16384 in
let ResourceLimits = {
  max_artifacts | Number,
  max_replay_runs | Number,
  max_artifact_bytes | Number,
  max_evaluator_bytes | Number,
  max_stderr_bytes | Number,
  max_path_bytes | Number,
  max_option_bytes | Number,
  max_diagnostic_bytes | Number,
  evaluator_timeout_milliseconds | Number,
  evaluator_poll_milliseconds | Number,
} in
{
  max_artifacts = MaximumArtifacts,
  max_replay_runs = MaximumReplayRuns,
  max_artifact_bytes = ArtifactMebibytes * MebibyteBytes,
  max_evaluator_bytes = EvaluatorMebibytes * MebibyteBytes,
  max_stderr_bytes = MebibyteBytes,
  max_path_bytes = MaximumPathBytes,
  max_option_bytes = MaximumOptionBytes,
  max_diagnostic_bytes = MebibyteBytes,
  evaluator_timeout_milliseconds = EvaluatorTimeoutMilliseconds,
  evaluator_poll_milliseconds = EvaluatorPollMilliseconds,
} | ResourceLimits
