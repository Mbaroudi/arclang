#!/bin/sh
# inside container: validate_in.sh <model dir> <aird name> <out dir>
set -u
src="$1"; aird="$2"; out="$3"
rm -rf "$out"; mkdir -p "$out/projects"; cp -R "$src" "$out/projects/M"
printf '<?xml version="1.0" encoding="UTF-8"?><projectDescription><name>M</name><comment></comment><projects></projects><buildSpec></buildSpec><natures><nature>org.polarsys.capella.project.nature</nature></natures></projectDescription>' > "$out/projects/M/.project"
timeout 900 xvfb-run -a -s "-screen 0 1920x1200x24" /opt/capella/capella -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid ${APPID:-org.polarsys.capella.core.validation.commandline} -data "$out/ws" -import "$out/projects/M" -input "/M/$aird" -outputfolder /M/validation -forceoutputfoldercreation -logfile "$out/log.html" > "$out/run.out" 2>&1
echo "exit=$?"
find "$out/projects/M/validation" -name validation-results.html
