#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
command=${1:-help}
if [[ $# -gt 0 ]]; then
  shift
fi

sandbox_name=${MINNAL_ONBOARDING_SANDBOX:-default}
sandbox_root_default="$repo_root/.tmp/onboarding/$sandbox_name"
sandbox_root=${MINNAL_ONBOARDING_DIR:-$sandbox_root_default}
minnal_home="$sandbox_root/home"
runtime_dir="$sandbox_root/runtime"
mobile_socket="$runtime_dir/minnal-mobile-sim.sock"

ensure_dirs() {
  mkdir -p "$minnal_home" "$runtime_dir"
}

run_in_sandbox() {
  ensure_dirs
  (
    cd "$repo_root"
    env \
      MINNAL_HOME="$minnal_home" \
      MINNAL_RUNTIME_DIR="$runtime_dir" \
      "$@"
  )
}


print_usage() {
  cat <<EOF
Usage: $(basename "$0") <command> [args...]

Commands:
  env                    Print the sandbox environment exports
  status                 Show sandbox paths and current contents
  reset                  Delete the sandbox entirely
  shell                  Open a clean shell with sandbox env vars set
  minnal [args...]        Run minnal inside the sandbox
  auth-status            Run 'minnal auth status' inside the sandbox
  fresh [args...]        Reset sandbox, then launch minnal with args
  login <provider> ...   Run 'minnal --provider <provider> login ...' in sandbox
  fixture-list           List saved local auth fixtures
  fixture-save <name>    Save current sandbox auth state as a local fixture
  fixture-load <name>    Load a saved auth fixture into this sandbox
  fixture-run <name> -- [args...]
                         Load a fixture, then run minnal with args
  mobile-start [scenario]
                         Start minnal-mobile-sim in background (default: onboarding)
  mobile-serve [scenario]
                         Run minnal-mobile-sim in foreground (default: onboarding)
  mobile-status          Show mobile simulator status
  mobile-state           Show full mobile simulator state
  mobile-reset           Reset the mobile simulator back to its initial scenario
  mobile-log             Show mobile simulator transition log
  help                   Show this help

Environment overrides:
  MINNAL_ONBOARDING_SANDBOX   Sandbox name (default: default)
  MINNAL_ONBOARDING_DIR       Explicit sandbox directory
  MINNAL_AUTH_FIXTURE_DIR     Fixture store (default: .tmp/auth-fixtures)

Examples:
  $(basename "$0") fresh
  $(basename "$0") login openai
  $(basename "$0") fixture-save normal-openai
  $(basename "$0") fixture-load normal-openai
  $(basename "$0") auth-status
  $(basename "$0") mobile-start onboarding
  $(basename "$0") mobile-status
EOF
}

print_env() {
  ensure_dirs
  cat <<EOF
export MINNAL_HOME="$minnal_home"
export MINNAL_RUNTIME_DIR="$runtime_dir"
EOF
}

status() {
  ensure_dirs
  echo "Sandbox name: $sandbox_name"
  echo "Sandbox root: $sandbox_root"
  echo "MINNAL_HOME:   $minnal_home"
  echo "RUNTIME_DIR:  $runtime_dir"
  echo

  if [[ -d "$minnal_home" ]]; then
    echo "Home contents:"
    find "$minnal_home" -maxdepth 3 \( -type f -o -type d \) | sed "s#^$sandbox_root#.#" | sort
  fi
  echo

  if [[ -S "$mobile_socket" ]]; then
    echo "Mobile simulator socket: $mobile_socket"
  else
    echo "Mobile simulator socket: not running"
  fi
}

reset() {
  rm -rf "$sandbox_root"
  echo "Removed onboarding sandbox: $sandbox_root"
}

open_shell() {
  ensure_dirs
  echo "Opening sandbox shell"
  echo "  MINNAL_HOME=$minnal_home"
  echo "  MINNAL_RUNTIME_DIR=$runtime_dir"
  env MINNAL_HOME="$minnal_home" MINNAL_RUNTIME_DIR="$runtime_dir" bash --noprofile --norc
}

run_minnal() {
  local binary_path="$repo_root/target/debug/minnal"
  if [[ -x "$binary_path" ]]; then
    run_in_sandbox "$binary_path" "$@"
  else
    run_in_sandbox cargo run --bin minnal -- "$@"
  fi
}

run_mobile_sim() {
  local binary_path="$repo_root/target/debug/minnal-mobile-sim"
  if [[ -x "$binary_path" ]]; then
    run_in_sandbox "$binary_path" "$@"
  else
    run_in_sandbox cargo run -p minnal-mobile-sim -- "$@"
  fi
}

run_auth_fixture() {
  MINNAL_ONBOARDING_SANDBOX="$sandbox_name" \
    MINNAL_ONBOARDING_DIR="$sandbox_root" \
    "$repo_root/scripts/auth_fixture.sh" "$@"
}

scenario_arg() {
  if [[ $# -gt 0 ]]; then
    printf '%s' "$1"
  else
    printf 'onboarding'
  fi
}

case "$command" in
  env)
    print_env
    ;;
  status)
    status
    ;;
  reset)
    reset
    ;;
  shell)
    open_shell
    ;;
  minnal|minnal)
    run_minnal "$@"
    ;;
  auth-status)
    run_minnal auth status
    ;;
  fresh)
    reset
    run_minnal "$@"
    ;;
  login)
    if [[ $# -lt 1 ]]; then
      echo "login requires a provider, for example: $(basename "$0") login openai" >&2
      exit 1
    fi
    provider=$1
    shift
    run_minnal --provider "$provider" login "$@"
    ;;
  fixture-list)
    run_auth_fixture list
    ;;
  fixture-save)
    run_auth_fixture save "$@"
    ;;
  fixture-load)
    run_auth_fixture load "$@"
    ;;
  fixture-run)
    run_auth_fixture run "$@"
    ;;
  mobile-start)
    scenario=$(scenario_arg "$@")
    run_mobile_sim start --scenario "$scenario"
    ;;
  mobile-serve)
    scenario=$(scenario_arg "$@")
    run_mobile_sim serve --scenario "$scenario"
    ;;
  mobile-status)
    run_mobile_sim status
    ;;
  mobile-state)
    run_mobile_sim state
    ;;
  mobile-reset)
    run_mobile_sim reset
    ;;
  mobile-log)
    run_mobile_sim log
    ;;
  help|-h|--help)
    print_usage
    ;;
  *)
    echo "Unknown command: $command" >&2
    echo >&2
    print_usage >&2
    exit 1
    ;;
esac
