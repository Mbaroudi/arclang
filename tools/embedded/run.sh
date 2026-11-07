#!/bin/sh
# Run a command in the embedded build image, the worktree mounted at /work,
# the cargo registry and target in named volumes (never in the worktree).
# No debug info and no incremental cache: the ~100 test binaries stay under
# ~10 GB and link within the Docker VM's memory.
set -e
here=$(cd "$(dirname "$0")/../.." && pwd)
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
    arclang-embedded:1 "$@"
