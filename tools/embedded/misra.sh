#!/bin/sh
# MISRA C 2012 (cppcheck's addon) on the C generated for every example with
# a state machine. Runs inside the embedded image:
#   tools/embedded/run.sh tools/embedded/misra.sh [arclang binary] [out dir]
# Prints, per rule, the violations over all examples, then the total.
# The host implementation of arclang_io.h (SIL only) is not checked.
set -u
bin=${1:-/target/debug/arclang}
out=${2:-/tmp/misra}
rm -rf "$out"; mkdir -p "$out"
n=0
for model in $(grep -rl --include=*.arc '^state_machine' examples | sort); do
    dir="$out/$(echo "$model" | tr '/' '_' | sed 's/\.arc$//')"
    if "$bin" export "$model" -f c -o "$dir" >/dev/null 2>&1; then
        n=$((n + 1))
        rm -rf "$dir/host"
        cppcheck --addon=misra --std=c99 --enable=style --inline-suppr --quiet \
            --suppress=missingIncludeSystem -I "$dir" "$dir"/*.c 2> "$dir/misra.txt" >/dev/null
    fi
done
echo "examples generated: $n"
cat "$out"/*/misra.txt | grep -o 'misra-c2012-[0-9]*\.[0-9]*' | sort | uniq -c | sort -k2 -V
echo "total: $(cat "$out"/*/misra.txt | grep -c 'misra-c2012-')"
