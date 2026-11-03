#!/bin/sh
# inside container: draw_in.sh <project dir (named as its model)> <work dir> <out dir>: Capella draws every machine, saved to <out dir>
set -u
src="$1"; work="$2"; outd="$3"
name=$(basename "$src")
rm -rf "$work"; mkdir -p "$work/projects"; cp -R "$src" "$work/projects/$name"
printf '<?xml version="1.0" encoding="UTF-8"?><projectDescription><name>%s</name><comment></comment><projects></projects><buildSpec></buildSpec><natures><nature>org.polarsys.capella.project.nature</nature></natures></projectDescription>' "$name" > "$work/projects/$name/.project"
timeout 900 xvfb-run -a -s "-screen 0 1920x1200x24" /opt/capella/capella -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid arclang.draw.machines -data "$work/ws" -import "$work/projects/$name" -input "/$name/$name.aird" > "$work/run.out" 2>&1
echo "exit=$?"; grep "^DRAW" "$work/run.out"
rm -rf "$outd"; mkdir -p "$outd"; cp "$work/projects/$name/$name".* "$outd/"
