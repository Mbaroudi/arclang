#!/bin/sh
# Run a command in the reverse image (Ghidra, on top of the formal image and its CBMC and
# Bitwuzla), the worktree mounted at /work, the cargo registry and target in
# the embedded image's named volumes (never in the worktree).
#   tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored
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
    arclang-embedded-reverse:1 "$@"
