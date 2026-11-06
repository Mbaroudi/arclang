#!/bin/sh
# ArcLang Studio's entry point.
#   (no argument)       VS Code in the browser on port 3000, the workspace opened.
#   arclang ...         the compiler (`docker run ... arclang-studio arclang check model.arc`).
#   any other command   run as given (a shell: `bash`).
#
# The server asks for a connection token: ARCLANG_STUDIO_TOKEN when set, else
# a random one, printed with the address to open. ARCLANG_STUDIO_TOKEN=none
# opens it without one: only behind something that authenticates (a reverse
# proxy with a login), or bound to 127.0.0.1.
#
# No trust prompt: the container is one user's, behind its token (VS Code
# trusts a dev container the same way).
set -e
if [ "$#" -gt 0 ]; then
    exec "$@"
fi
port=${ARCLANG_STUDIO_PORT:-3000}
folder=${ARCLANG_STUDIO_FOLDER:-/home/arclang/workspace}
if [ -z "$(ls -A "$folder" 2>/dev/null)" ]; then
    # Nothing mounted: the examples, to start from.
    folder=/home/arclang/examples
fi
case "${ARCLANG_STUDIO_TOKEN:-}" in
    none)
        echo "ArcLang Studio: http://localhost:$port/?folder=$folder (no connection token)"
        exec openvscode-server --host 0.0.0.0 --port "$port" --without-connection-token --disable-workspace-trust --default-folder "$folder"
        ;;
    "")
        token=$(head -c 24 /dev/urandom | od -An -tx1 | tr -d ' \n')
        ;;
    *)
        token=$ARCLANG_STUDIO_TOKEN
        ;;
esac
echo "ArcLang Studio: http://localhost:$port/?tkn=$token&folder=$folder"
exec openvscode-server --host 0.0.0.0 --port "$port" --connection-token "$token" --disable-workspace-trust --default-folder "$folder"
