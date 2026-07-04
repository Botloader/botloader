#!/bin/bash
set -e
cd "${0%/*}"

usage() {
    echo "usage: $0 [vm|full] <light|medium|heavy|all> [extra vmbench args...]"
    echo
    echo "modes:"
    echo "  vm      (default) in-process vm creation, no infrastructure needed"
    echo "  full    whole scheduler flow with real vm worker processes,"
    echo "          needs DATABASE_URL + DISCORD_BOT_TOKEN/DISCORD_CLIENT_ID/DISCORD_CLIENT_SECRET"
    echo "          (picked up from the repo root .env if not already set)"
    echo
    echo "variants:"
    echo "  light   1 small script"
    echo "  medium  5 medium sized scripts"
    echo "  heavy   15 large scripts"
    echo "  all     run all three variants after each other"
    echo
    echo "extra args are passed to vmbench, e.g: $0 medium --iterations 50 --quiet"
    exit 1
}

mode="vm"
case "$1" in
    vm|full)
        mode="$1"
        shift
        ;;
esac

full_env_vars=(DATABASE_URL DISCORD_BOT_TOKEN DISCORD_CLIENT_ID DISCORD_CLIENT_SECRET)

# import only the vars the full bench needs from the repo root .env, so the
# rest of it (RUST_LOG etc) doesn't affect the bench
load_env() {
    local env_file="../../.env"
    [[ -f "$env_file" ]] || return 0
    eval "$(
        set -a
        # shellcheck source=/dev/null
        source "$env_file" >/dev/null
        for var in "${full_env_vars[@]}"; do
            [[ -n "${!var:-}" ]] && printf 'export %s=%q\n' "$var" "${!var}"
        done
    )"
}

prepare_full() {
    for var in "${full_env_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            load_env
            break
        fi
    done

    missing=()
    for var in "${full_env_vars[@]}"; do
        [[ -z "${!var:-}" ]] && missing+=("$var")
    done
    if ((${#missing[@]})); then
        echo "error: full mode needs ${missing[*]}" >&2
        echo "set them in the environment or in the repo root .env" >&2
        exit 1
    fi

    # the bench takes over the scheduler worker socket; refuse to hijack a
    # running scheduler's workers
    local socket=/tmp/botloader_scheduler_workers
    if command -v ss >/dev/null && ss -xl 2>/dev/null | grep -qF "$socket"; then
        echo "error: something is already listening on $socket (a running scheduler?)" >&2
        echo "the full bench would take over its worker socket, stop it first" >&2
        exit 1
    fi
}

variant="${1:-}"

# build without spamming warnings: hide cargo output unless the build fails,
# and run the built binary directly so cargo doesn't replay cached warnings
vmbench_bin="${CARGO_TARGET_DIR:-../../target}/release/vmbench"

build() {
    echo "building vmbench (release)..."
    local out
    if ! out=$(cargo build --release -p vmbench 2>&1); then
        printf '%s\n' "$out" >&2
        exit 1
    fi
}

run_variant() {
    echo
    echo "=== mode: $mode, variant: $1 ==="
    "$vmbench_bin" "$mode" "${@:2}" scripts/"$1"/*.ts
}

case "$variant" in
    light|medium|heavy)
        shift
        [[ "$mode" == full ]] && prepare_full
        build
        run_variant "$variant" "$@"
        ;;
    all)
        shift
        [[ "$mode" == full ]] && prepare_full
        build
        for v in light medium heavy; do
            run_variant "$v" "$@"
        done
        ;;
    *)
        usage
        ;;
esac
