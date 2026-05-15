//! Telemetry collection is intentionally disabled.
//!
//! The rest of the application still calls these functions at lifecycle boundaries.
//! Keeping a no-op compatibility surface avoids threading feature-specific conditionals through
//! the agent, TUI, and server code while ensuring no telemetry state is recorded or sent.

#[derive(Debug, Clone, Copy)]
pub enum SessionEndReason {
    NormalExit,
    Panic,
    Signal,
    Disconnect,
    Reload,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    ProviderTimeout,
    AuthFailed,
    ToolError,
    McpError,
    RateLimited,
}

pub fn is_enabled() -> bool {
    false
}

pub fn record_setup_step_once(_step: &'static str) {}

pub fn record_feedback(_text: &str) {}

pub fn record_install_if_first_run() {}

pub fn record_upgrade_if_needed() {}

pub fn record_provider_selected(_provider: &str) {}

pub fn record_auth_started(provider: &str, method: &str) {
    crate::logging::auth_event("auth_started", provider, &[("method", method)]);
}

pub fn record_auth_failed(provider: &str, method: &str) {
    record_auth_failed_reason(provider, method, "unknown");
}

pub fn record_auth_failed_reason(provider: &str, method: &str, reason: &str) {
    crate::logging::auth_event(
        "auth_failed",
        provider,
        &[("method", method), ("reason", reason)],
    );
}

pub fn record_auth_cancelled(provider: &str, method: &str) {
    crate::logging::auth_event("auth_cancelled", provider, &[("method", method)]);
}

pub fn record_auth_surface_blocked(provider: &str, method: &str) {
    crate::logging::auth_event("auth_surface_blocked", provider, &[("method", method)]);
}

pub fn record_auth_surface_blocked_reason(provider: &str, method: &str, reason: &str) {
    crate::logging::auth_event(
        "auth_surface_blocked",
        provider,
        &[("method", method), ("reason", reason)],
    );
}

pub fn record_auth_success(provider: &str, method: &str) {
    crate::logging::auth_event("auth_success", provider, &[("method", method)]);
}

pub fn begin_session(_provider: &str, _model: &str) {}

pub fn begin_session_with_parent(
    _provider: &str,
    _model: &str,
    _parent_session_id: Option<String>,
    _resumed_session: bool,
) {
}

pub fn begin_resumed_session(_provider: &str, _model: &str) {}

pub fn record_turn() {}

pub fn record_command_family(_command: &str) {}

pub fn record_assistant_response() {}

pub fn record_memory_injected(_count: usize, _age_ms: u64) {}

pub fn record_tool_call() {}

pub fn record_tool_failure() {}

pub fn record_connection_type(_connection: &str) {}

pub fn record_token_usage(
    _input_tokens: u64,
    _output_tokens: u64,
    _cache_read_input_tokens: Option<u64>,
    _cache_creation_input_tokens: Option<u64>,
) {
}

pub fn record_error(_category: ErrorCategory) {}

pub fn record_provider_switch() {}

pub fn record_model_switch() {}

pub fn record_user_cancelled() {}

pub fn record_tool_execution(
    _name: &str,
    _input: &serde_json::Value,
    _succeeded: bool,
    _latency_ms: u64,
) {
}

pub fn end_session(_provider_end: &str, _model_end: &str) {}

pub fn end_session_with_reason(_provider_end: &str, _model_end: &str, _reason: SessionEndReason) {}

pub fn record_crash(_provider_end: &str, _model_end: &str, _reason: SessionEndReason) {}

pub fn current_provider_model() -> Option<(String, String)> {
    None
}
