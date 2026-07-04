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

variant="${1:-}"

run_variant() {
    echo
    echo "=== mode: $mode, variant: $1 ==="
    cargo run --release -p vmbench -- "$mode" "${@:2}" scripts/"$1"/*.ts
}

case "$variant" in
    light|medium|heavy)
        shift
        # build up front: in full mode the workers are spawned from the built binary
        cargo build --release -p vmbench
        run_variant "$variant" "$@"
        ;;
    all)
        shift
        cargo build --release -p vmbench
        for v in light medium heavy; do
            run_variant "$v" "$@"
        done
        ;;
    *)
        usage
        ;;
esac
