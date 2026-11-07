#!/bin/sh
# inside container: csc_in.sh <project dir (named as its model)> <config.properties> <work dir> <out .m>
set -u
src="$1"; cfg="$2"; work="$3"; out="$4"
name=$(basename "$src")
rm -rf "$work"; mkdir -p "$work/projects"; cp -R "$src" "$work/projects/$name"
printf '<?xml version="1.0" encoding="UTF-8"?><projectDescription><name>%s</name><comment></comment><projects></projects><buildSpec></buildSpec><natures><nature>org.polarsys.capella.project.nature</nature></natures></projectDescription>' "$name" > "$work/projects/$name/.project"
imp="-import $work/projects/$name"
if [ "${MIGRATE:-0}" = 1 ]; then
  timeout 900 xvfb-run -a -s "-screen 0 1920x1200x24" /opt/capella/capella -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid org.polarsys.capella.migration -data "$work/ws" $imp -input "/$name" > "$work/migration.out" 2>&1
  echo "migration exit=$?"
  imp=""
fi
CSC_CONFIG="$cfg" CSC_OUT="$out" CSC_MLCORE="${out%.m}.mlcore" timeout 900 xvfb-run -a -s "-screen 0 1920x1200x24" /opt/capella/capella -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid arclang.csc.run -data "$work/ws" $imp -input "/$name/$name.aird" > "$work/run.out" 2>&1
echo "exit=$? $(basename "$out") $(wc -l < "$out" 2>/dev/null) lines"
