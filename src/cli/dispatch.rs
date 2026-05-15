#![cfg_attr(test, allow(clippy::await_holding_lock))]

use anyhow::Result;
use std::process::{Command as ProcessCommand, Stdio};
use std::time::Instant;

use super::args::{
    AmbientCommand, Args, AuthCommand, Command, MemoryCommand, ModelCommand, ProviderCommand,
    RestartCommand, SessionCommand, TranscriptModeArg,
};
use crate::{
    agent, auth, build, provider, provider_catalog, server, session, setup_hints, startup_profile,
    tui,
};

use super::{
    commands, debug, hot_exec, login, output, provider_init, selfdev, terminal, tui_launch,
};
use provider_init::{ProviderChoice, ProviderSelection};

pub(crate) async fn run_main(mut args: Args) -> Result<()> {
    resolve_resume_arg(&mut args)?;
    let provider_selection = provider_init::resolve_provider_selection(
        &args.provider,
        args.provider_profile.as_deref(),
    )?;
    provider_selection.apply_env()?;

    match args.command {
        Some(Command::Serve {
            temporary_server,
            owner_pid,
            temp_idle_timeout_secs,
        }) => {
            let serve_start = Instant::now();
            crate::env::set_var("MINNAL_NON_INTERACTIVE", "1");
            if temporary_server {
                server::configure_temporary_server(owner_pid, temp_idle_timeout_secs);
            }
            let provider_start = Instant::now();
            let provider =
                provider_init::init_provider(provider_selection.choice(), args.model.as_deref())
                    .await?;
            maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
            let provider_ms = provider_start.elapsed().as_millis();
            let server_new_start = Instant::now();
            let server = server::Server::new(provider);
            let server_new_ms = server_new_start.elapsed().as_millis();
            crate::logging::info(&format!(
                "[TIMING] serve bootstrap: provider_init={}ms, server_new={}ms, before_run={}ms",
                provider_ms,
                server_new_ms,
                serve_start.elapsed().as_millis()
            ));
            server.run().await?;
        }
        Some(Command::Connect) => {
            tui_launch::run_client().await?;
        }
        Some(Command::Run {
            message,
            json,
            ndjson,
        }) => {
            commands::run_single_message_command(
                provider_selection.choice(),
                args.model.as_deref(),
                args.resume.as_deref(),
                &message,
                json,
                ndjson,
            )
            .await?;
            maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
        }
        Some(Command::Login {
            account,
            no_browser,
            print_auth_url,
            callback_url,
            auth_code,
            json,
            complete,
            no_validate,
            google_access_tier,
            api_base,
            api_key,
            api_key_env,
        }) => {
            login::run_login(
                provider_selection.choice(),
                account.as_deref(),
                login::LoginOptions {
                    no_browser,
                    print_auth_url,
                    callback_url,
                    auth_code,
                    json,
                    complete,
                    no_validate,
                    google_access_tier: google_access_tier.map(|tier| match tier {
                        super::args::GoogleAccessTierArg::Full => {
                            auth::google::GmailAccessTier::Full
                        }
                        super::args::GoogleAccessTierArg::Readonly => {
                            auth::google::GmailAccessTier::ReadOnly
                        }
                    }),
                    openai_compatible_api_base: api_base,
                    openai_compatible_api_key: api_key,
                    openai_compatible_api_key_env: api_key_env,
                    openai_compatible_default_model: args.model.clone(),
                },
            )
            .await?;
        }
        Some(Command::Repl) => {
            let (provider, registry) = provider_init::init_provider_and_registry(
                provider_selection.choice(),
                args.model.as_deref(),
            )
            .await?;
            maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
            let mut agent = agent::Agent::new(provider, registry);
            agent.repl().await?;
        }
        Some(Command::Update) => {
            hot_exec::run_update()?;
        }
        Some(Command::Version { json }) => {
            commands::run_version_command(json)?;
        }
        Some(Command::Usage { json }) => {
            commands::run_usage_command(json).await?;
        }
        Some(Command::SelfDev { build }) => {
            selfdev::run_self_dev(build, args.resume).await?;
        }
        Some(Command::Debug {
            command,
            arg,
            session,
            socket,
            wait,
        }) => {
            debug::run_debug_command(&command, &arg, session, socket, wait).await?;
        }
        Some(Command::Auth(subcmd)) => match subcmd {
            AuthCommand::Status { json } => commands::run_auth_status_command(json)?,
            AuthCommand::Doctor {
                provider,
                validate,
                json,
            } => {
                let provider_arg =
                    auth_doctor_provider_arg(provider.as_deref(), &provider_selection);
                commands::run_auth_doctor_command(provider_arg, validate, json).await?
            }
        },
        Some(Command::Provider(subcmd)) => match subcmd {
            ProviderCommand::List { json } => {
                commands::run_provider_list_command(json)?;
            }
            ProviderCommand::Current { json } => {
                commands::run_provider_current_command(
                    provider_selection.choice(),
                    args.model.as_deref(),
                    Some(provider_selection.provider_id()),
                    json,
                )
                .await?;
                maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
            }
            ProviderCommand::Add {
                name,
                base_url,
                model,
                context_window,
                api_key_env,
                api_key,
                api_key_stdin,
                no_api_key,
                auth,
                auth_header,
                env_file,
                set_default,
                overwrite,
                provider_routing,
                model_catalog,
                json,
            } => {
                commands::run_provider_add_command(commands::ProviderAddOptions {
                    name,
                    base_url,
                    model,
                    context_window,
                    api_key_env,
                    api_key,
                    api_key_stdin,
                    no_api_key,
                    auth,
                    auth_header,
                    env_file,
                    set_default,
                    overwrite,
                    provider_routing,
                    model_catalog,
                    json,
                })?;
            }
        },
        Some(Command::Memory(subcmd)) => {
            commands::run_memory_command(map_memory_subcommand(subcmd))?;
        }
        Some(Command::Session(subcmd)) => match subcmd {
            SessionCommand::Rename {
                session,
                name,
                clear,
                json,
            } => commands::run_session_rename_command(&session, name.as_deref(), clear, json)?,
        },
        Some(Command::Ambient(subcmd)) => {
            commands::run_ambient_command(map_ambient_subcommand(subcmd)).await?;
        }
        Some(Command::Pair { list, revoke }) => {
            commands::run_pair_command(list, revoke)?;
        }
        Some(Command::Permissions) => {
            tui::permissions::run_permissions()?;
        }
        Some(Command::Transcript {
            text,
            mode,
            session,
        }) => {
            commands::run_transcript_command(text, map_transcript_mode(mode), session).await?;
        }
        Some(Command::Dictate { r#type }) => {
            commands::run_dictate_command(r#type).await?;
        }
        Some(Command::SetupHotkey {
            listen_macos_hotkey,
        }) => {
            setup_hints::run_setup_hotkey(listen_macos_hotkey)?;
        }
        Some(Command::SetupLauncher) => {
            setup_hints::run_setup_launcher()?;
        }
        Some(Command::Browser { action }) => {
            commands::run_browser(&action).await?;
        }
        Some(Command::Replay {
            session,
            swarm,
            export,
            speed,
            timeline,
            auto_edit,
            video,
            cols,
            rows,
            fps,
            centered,
            no_centered,
        }) => {
            let centered_override = if centered {
                Some(true)
            } else if no_centered {
                Some(false)
            } else {
                None
            };
            tui_launch::run_replay_command(
                &session,
                swarm,
                export,
                auto_edit,
                speed,
                timeline.as_deref(),
                video.as_deref(),
                cols,
                rows,
                fps,
                centered_override,
            )
            .await?;
        }
        Some(Command::Model(subcmd)) => match subcmd {
            ModelCommand::List { json, verbose } => {
                commands::run_model_command(
                    provider_selection.choice(),
                    args.model.as_deref(),
                    json,
                    verbose,
                )
                .await?;
                maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
            }
        },
        Some(Command::AuthTest {
            login,
            all_configured,
            no_smoke,
            no_tool_smoke,
            prompt,
            json,
            output,
        }) => {
            commands::run_auth_test_command(
                provider_selection.choice(),
                args.model.as_deref(),
                login,
                all_configured,
                no_smoke,
                no_tool_smoke,
                prompt.as_deref(),
                json,
                output.as_deref(),
            )
            .await?;
            maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
        }
        Some(Command::Restart { action }) => match action {
            RestartCommand::Save { auto_restore } => {
                commands::run_restart_save_command(auto_restore).await?
            }
            RestartCommand::Restore => commands::run_restart_restore_command()?,
            RestartCommand::Status => commands::run_restart_status_command()?,
            RestartCommand::Clear => commands::run_restart_clear_command()?,
        },
        None => run_default_command(args, provider_selection).await?,
    }

    Ok(())
}

fn auth_doctor_provider_arg<'a>(
    positional_provider: Option<&'a str>,
    global_provider: &'a ProviderSelection,
) -> Option<&'a str> {
    positional_provider.or_else(|| {
        if global_provider.is_auto() {
            None
        } else {
            Some(global_provider.provider_id())
        }
    })
}

fn maybe_persist_cli_provider_model(provider_selection: &ProviderSelection, model: Option<&str>) {
    let provider = (!provider_selection.is_auto()).then_some(provider_selection.provider_id());
    let model = model.map(str::trim).filter(|value| !value.is_empty());
    let result = match (provider, model) {
        (Some(provider), Some(model)) => {
            crate::config::Config::set_default_model(Some(model), Some(provider))
        }
        (Some(provider), None) => crate::config::Config::set_default_provider(Some(provider)),
        (None, Some(model)) => {
            if let Some((model, provider)) = provider_model_default_from_prefixed_model(model) {
                crate::config::Config::set_default_model(Some(&model), Some(&provider))
            } else {
                crate::config::Config::set_default_model_only(Some(model))
            }
        }
        (None, None) => return,
    };
    if let Err(err) = result {
        crate::logging::warn(&format!(
            "Failed to persist CLI provider/model selection: {}",
            err
        ));
    }
}

fn provider_model_default_from_prefixed_model(model: &str) -> Option<(String, String)> {
    let (prefix, rest) = model.split_once(':')?;
    let prefix = prefix.trim();
    let rest = rest.trim();
    if prefix.is_empty() || rest.is_empty() {
        return None;
    }
    if crate::config::config().providers.contains_key(prefix) {
        return Some((rest.to_string(), prefix.to_string()));
    }
    if let Some(profile) = crate::provider_catalog::openai_compatible_profile_by_id(prefix) {
        return Some((rest.to_string(), profile.id.to_string()));
    }
    provider_key_for_model_prefix(prefix).map(|provider| (model.to_string(), provider.to_string()))
}

fn provider_key_for_model_prefix(prefix: &str) -> Option<&'static str> {
    match prefix.trim().to_ascii_lowercase().as_str() {
        "claude" | "anthropic" => Some("claude"),
        "openai" => Some("openai"),
        "copilot" => Some("copilot"),
        "antigravity" => Some("antigravity"),
        "gemini" => Some("gemini"),
        "cursor" => Some("cursor"),
        "bedrock" => Some("bedrock"),
        "openrouter" => Some("openrouter"),
        _ => None,
    }
}

fn resolve_resume_arg(args: &mut Args) -> Result<()> {
    if let Some(ref resume_id) = args.resume {
        if resume_id.is_empty() {
            return tui_launch::list_sessions();
        }

        match resolve_resume_id(resume_id) {
            Ok(full_id) => {
                args.resume = Some(full_id);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                if !output::quiet_enabled() {
                    eprintln!("\nUse `minnal --resume` to list available sessions.");
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn resolve_resume_id(resume_id: &str) -> Result<String> {
    match session::find_session_by_name_or_id(resume_id) {
        Ok(full_id) => Ok(full_id),
        Err(native_err) => match crate::import::import_external_resume_id(resume_id)? {
            Some(imported_id) => Ok(imported_id),
            None => Err(native_err),
        },
    }
}

fn map_memory_subcommand(subcmd: MemoryCommand) -> commands::MemorySubcommand {
    match subcmd {
        MemoryCommand::List { scope, tag } => commands::MemorySubcommand::List { scope, tag },
        MemoryCommand::Search { query, semantic } => {
            commands::MemorySubcommand::Search { query, semantic }
        }
        MemoryCommand::Export { output, scope } => {
            commands::MemorySubcommand::Export { output, scope }
        }
        MemoryCommand::Import {
            input,
            scope,
            overwrite,
        } => commands::MemorySubcommand::Import {
            input,
            scope,
            overwrite,
        },
        MemoryCommand::Stats => commands::MemorySubcommand::Stats,
        MemoryCommand::ClearTest => commands::MemorySubcommand::ClearTest,
    }
}

fn map_ambient_subcommand(subcmd: AmbientCommand) -> commands::AmbientSubcommand {
    match subcmd {
        AmbientCommand::Status => commands::AmbientSubcommand::Status,
        AmbientCommand::Log => commands::AmbientSubcommand::Log,
        AmbientCommand::Trigger => commands::AmbientSubcommand::Trigger,
        AmbientCommand::Stop => commands::AmbientSubcommand::Stop,
        AmbientCommand::RunVisible => commands::AmbientSubcommand::RunVisible,
    }
}

fn map_transcript_mode(mode: TranscriptModeArg) -> crate::protocol::TranscriptMode {
    match mode {
        TranscriptModeArg::Insert => crate::protocol::TranscriptMode::Insert,
        TranscriptModeArg::Append => crate::protocol::TranscriptMode::Append,
        TranscriptModeArg::Replace => crate::protocol::TranscriptMode::Replace,
        TranscriptModeArg::Send => crate::protocol::TranscriptMode::Send,
    }
}

async fn run_default_command(args: Args, provider_selection: ProviderSelection) -> Result<()> {
    startup_profile::mark("run_main_none_branch");

    let explicit_provider_or_model =
        !provider_selection.is_auto() || args.model.is_some() || args.provider_profile.is_some();
    if args.resume.is_none()
        && !explicit_provider_or_model
        && commands::maybe_run_pending_restart_restore_on_startup().await?
    {
        return Ok(());
    }

    let startup_hints = if args.fresh_spawn {
        None
    } else {
        setup_hints::maybe_show_setup_hints()
    };
    startup_profile::mark("setup_hints");

    if args.resume.is_none() {
        terminal::show_crash_resume_hint();
    }
    startup_profile::mark("crash_resume_hint");

    let cwd = std::env::current_dir()?;
    let in_minnal_repo = build::is_minnal_repo(&cwd);
    startup_profile::mark("is_minnal_repo");
    let already_in_selfdev = crate::cli::selfdev::client_selfdev_requested();

    if in_minnal_repo && !already_in_selfdev && !args.no_selfdev {
        output::stderr_info("📍 Detected minnal repository - enabling self-dev mode");
        output::stderr_info("   Using shared server with self-dev session mode");
        output::stderr_info("   (use --no-selfdev to disable auto-detection)");
        output::stderr_blank_line();

        crate::env::set_var(selfdev::CLIENT_SELFDEV_ENV, "1");
        crate::process_title::set_initial_title(&args);
    }

    startup_profile::mark("client_mode_start");
    let mut server_running = if args.fresh_spawn {
        true
    } else {
        server_is_running().await
    };
    startup_profile::mark("server_check");

    if !server_running {
        server_running = wait_for_existing_reload_server("client startup").await;
    }

    if !server_running && std::env::var("MINNAL_RESUMING").is_ok() {
        server_running = wait_for_resuming_server(
            "client startup without reload marker",
            std::time::Duration::from_secs(5),
        )
        .await;
    }

    if server_running && explicit_provider_or_model {
        output::stderr_info(
            "Server already running; provider/model flags only apply when starting a new server.",
        );
        maybe_persist_cli_provider_model(&provider_selection, args.model.as_deref());
        output::stderr_info(format!(
            "Current server settings control `/model`. Restart server to apply: --provider {}{}",
            provider_selection.provider_id(),
            args.model
                .as_ref()
                .map(|m| format!(" --model {}", m))
                .unwrap_or_default()
        ));
    }

    if !server_running {
        maybe_prompt_server_bootstrap_login(provider_selection.choice()).await?;
        spawn_server(&provider_selection, args.model.as_deref()).await?;
    }

    startup_profile::mark("pre_tui_client");
    if std::env::var("MINNAL_RESUMING").is_err() && server_running {
        output::stderr_info("Connecting to server...");
    }
    tui_launch::run_tui_client(
        args.resume,
        startup_hints,
        !server_running,
        args.fresh_spawn,
    )
    .await?;

    Ok(())
}

pub(crate) async fn server_is_running() -> bool {
    server_is_running_at(&server::socket_path()).await
}

async fn wait_for_existing_reload_server(context: &str) -> bool {
    if let Some(state) = server::recent_reload_state(std::time::Duration::from_secs(30)) {
        match state.phase {
            server::ReloadPhase::Starting => {
                crate::logging::info(&format!(
                    "Reload state=starting during {}; waiting for existing server to return",
                    context
                ));
                return wait_for_reloading_server().await;
            }
            server::ReloadPhase::Failed => {
                crate::logging::warn(&format!(
                    "Reload state=failed during {} on {}: {}; recent_state={}",
                    context,
                    server::socket_path().display(),
                    state
                        .detail
                        .unwrap_or_else(|| "unknown reload failure".to_string()),
                    server::reload_state_summary(std::time::Duration::from_secs(60))
                ));
            }
            server::ReloadPhase::SocketReady => {}
        }
    }

    false
}

pub(crate) async fn wait_for_resuming_server(context: &str, timeout: std::time::Duration) -> bool {
    let socket_path = server::socket_path();
    let start = std::time::Instant::now();
    let mut announced = false;

    while start.elapsed() < timeout {
        if server_is_running_at(&socket_path).await {
            crate::logging::info(&format!(
                "Server became available during resume wait for {} after {}ms",
                context,
                start.elapsed().as_millis()
            ));
            return true;
        }

        if !announced {
            crate::logging::info(&format!(
                "Server not ready during {}; waiting up to {}ms for a resumed/reloading server before spawning a replacement",
                context,
                timeout.as_millis()
            ));
            announced = true;
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    false
}

pub(crate) async fn wait_for_reloading_server() -> bool {
    match server::await_reload_handoff(&server::socket_path(), std::time::Duration::from_secs(30))
        .await
    {
        server::ReloadWaitStatus::Ready => true,
        server::ReloadWaitStatus::Failed(detail) => {
            crate::logging::warn(&format!(
                "Reload handoff failed while waiting for server on {}: {}; recent_state={}",
                server::socket_path().display(),
                detail.unwrap_or_else(|| "unknown reload failure".to_string()),
                server::reload_state_summary(std::time::Duration::from_secs(60))
            ));
            false
        }
        server::ReloadWaitStatus::Idle => false,
        server::ReloadWaitStatus::Waiting { .. } => false,
    }
}

async fn server_is_running_at(path: &std::path::Path) -> bool {
    server::is_server_ready(path).await || server::has_live_listener(path).await
}

#[cfg(unix)]
fn spawn_lock_path(socket_path: &std::path::Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.spawning", socket_path.display()))
}

#[cfg(unix)]
struct SpawnLockGuard {
    _file: std::fs::File,
    path: std::path::PathBuf,
}

#[cfg(unix)]
impl Drop for SpawnLockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(unix)]
fn try_acquire_spawn_lock(path: &std::path::Path) -> Result<Option<SpawnLockGuard>> {
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)?;
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if ret == 0 {
        Ok(Some(SpawnLockGuard {
            _file: file,
            path: path.to_path_buf(),
        }))
    } else {
        Ok(None)
    }
}

#[cfg(unix)]
async fn acquire_spawn_lock_or_wait(
    socket_path: &std::path::Path,
) -> Result<Option<SpawnLockGuard>> {
    let lock_path = spawn_lock_path(socket_path);
    let wait_start = std::time::Instant::now();
    let wait_timeout = std::time::Duration::from_secs(10);
    let mut announced_wait = false;

    loop {
        if let Some(lock) = try_acquire_spawn_lock(&lock_path)? {
            return Ok(Some(lock));
        }

        if server_is_running_at(socket_path).await {
            return Ok(None);
        }

        if !announced_wait {
            output::stderr_info("Another client is starting the server, waiting...");
            announced_wait = true;
        }

        if wait_start.elapsed() >= wait_timeout {
            anyhow::bail!(
                "Timed out waiting for another client to start server at {}",
                socket_path.display()
            );
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub(crate) async fn maybe_prompt_server_bootstrap_login(
    provider_choice: &ProviderChoice,
) -> Result<()> {
    startup_profile::mark("cred_check_start");
    let mut cred_state = detect_bootstrap_credentials().await;
    startup_profile::mark("cred_check_done");

    if !cred_state.has_any
        && auth::AuthStatus::has_any_untrusted_external_auth()
        && *provider_choice == ProviderChoice::Auto
    {
        let _ = provider_init::maybe_run_external_auth_auto_import_flow().await?;
        cred_state = detect_bootstrap_credentials().await;
    }

    if !cred_state.has_any && *provider_choice == ProviderChoice::Auto {
        let provider = provider_init::prompt_login_provider_selection(
            &provider_catalog::server_bootstrap_login_providers(),
            "No credentials found. Let's log in!\n\nChoose a provider:",
        )?;
        login::run_login_provider(provider, None, login::LoginOptions::default()).await?;
        provider_init::apply_login_provider_profile_env(provider);
        output::stderr_blank_line();
    }

    Ok(())
}

struct BootstrapCredentialState {
    has_any: bool,
}

async fn detect_bootstrap_credentials() -> BootstrapCredentialState {
    let (has_claude, has_openai) = tokio::join!(
        tokio::task::spawn_blocking(auth::claude::has_api_key),
        tokio::task::spawn_blocking(|| auth::codex::load_credentials().is_ok()),
    );
    let has_claude = has_claude.unwrap_or(false);
    let has_openai = has_openai.unwrap_or(false);
    let has_openrouter = provider::openrouter::OpenRouterProvider::has_credentials();
    let has_copilot = auth::copilot::has_copilot_credentials();

    BootstrapCredentialState {
        has_any: has_claude || has_openai || has_openrouter || has_copilot,
    }
}

pub(crate) async fn spawn_server(
    provider_selection: &ProviderSelection,
    model: Option<&str>,
) -> Result<()> {
    let socket_path = server::socket_path();
    if server_is_running_at(&socket_path).await {
        startup_profile::mark("server_ready");
        return Ok(());
    }

    if wait_for_existing_reload_server("server spawn").await {
        startup_profile::mark("server_ready");
        return Ok(());
    }

    #[cfg(unix)]
    let _spawn_lock = acquire_spawn_lock_or_wait(&socket_path).await?;

    if server_is_running_at(&socket_path).await {
        startup_profile::mark("server_ready");
        return Ok(());
    }

    if wait_for_existing_reload_server("server spawn after lock").await {
        startup_profile::mark("server_ready");
        return Ok(());
    }

    startup_profile::mark("server_spawn_start");
    output::stderr_info("Starting server...");
    let client_requested_selfdev = selfdev::client_selfdev_requested();
    let exe = build::shared_server_update_candidate(client_requested_selfdev)
        .map(|(path, _)| path)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| anyhow::anyhow!("Could not determine executable path for server spawn"))?;
    let mut cmd = ProcessCommand::new(&exe);
    cmd.env_remove(selfdev::CLIENT_SELFDEV_ENV);
    if client_requested_selfdev {
        cmd.env("MINNAL_DEBUG_CONTROL", "1");
    }
    cmd.arg("--provider").arg(provider_selection.provider_id());
    if let Some(model) = model {
        cmd.arg("--model").arg(model);
    }
    cmd.arg("serve")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        let _child = server::spawn_server_notify(&mut cmd).await?;
        startup_profile::mark("server_ready");
    }
    #[cfg(not(unix))]
    {
        use std::io::Read;

        let mut child = cmd.spawn()?;
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(5);
        while start.elapsed() < timeout {
            if crate::transport::is_socket_path(&server::socket_path()) {
                if crate::transport::Stream::connect(server::socket_path())
                    .await
                    .is_ok()
                {
                    startup_profile::mark("server_ready");
                    return Ok(());
                }
            }

            if let Some(status) = child.try_wait()? {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                let detail = stderr.trim();
                if detail.is_empty() {
                    anyhow::bail!("Server exited before becoming ready (status: {})", status);
                }
                anyhow::bail!(
                    "Server exited before becoming ready (status: {}). {}",
                    status,
                    detail
                );
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        anyhow::bail!(
            "Timed out waiting for server to become ready at {} after {}ms",
            server::socket_path().display(),
            timeout.as_millis()
        );
    }

    Ok(())
}

#[cfg(test)]
#[path = "dispatch_tests.rs"]
mod dispatch_tests;
