#!/bin/sh
# Run a command in the SoC image (Zephyr, Renode, on top of the embedded
# image), the worktree mounted at /work, the cargo registry and target in
# the embedded image's named volumes (never in the worktree).
#   tools/embedded/soc/run.sh cargo run -- pil examples/energy/battery-charger/battery_charger.arc
set -e
here=$(cd "$(dirname "$0")/../../.." && pwd)
exec docker run --rm -i \
    -v "$here":/work \
    -v arclang-embedded-cargo:/usr/local/cargo/registry \
    -v arclang-embedded-target:/target \
    -e CARGO_TARGET_DIR=/target \
    -e CARGO_PROFILE_DEV_DEBUG=0 \
    -e CARGO_PROFILE_TEST_DEBUG=0 \
    -e CARGO_INCREMENTAL=0 \
    -e CARGO_BUILD_JOBS=${ARCLANG_JOBS:-4} \
    -w /work \
    ${ARCLANG_DOCKER_ARGS:-} \
    arclang-embedded-soc:1 "$@"
