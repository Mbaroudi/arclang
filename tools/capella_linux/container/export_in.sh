#!/bin/sh
# inside container: export_in.sh <model dir> <aird name> <out dir>  -> PNG of every representation in <out>/projects/M/images
set -u
src="$1"; aird="$2"; out="$3"
rm -rf "$out"; mkdir -p "$out/projects"; cp -R "$src" "$out/projects/M"
printf '<?xml version="1.0" encoding="UTF-8"?><projectDescription><name>M</name><comment></comment><projects></projects><buildSpec></buildSpec><natures><nature>org.polarsys.capella.project.nature</nature></natures></projectDescription>' > "$out/projects/M/.project"
timeout 900 xvfb-run -a -s "-screen 0 1920x1200x24" /opt/capella/capella -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid org.polarsys.capella.exportRepresentations -data "$out/ws" -import "$out/projects/M" -input "/M/$aird" -outputfolder /M/images -forceoutputfoldercreation -imageFormat PNG > "$out/run.out" 2>&1
echo "exit=$?"
find "$out/projects/M/images" -type f | head -50
