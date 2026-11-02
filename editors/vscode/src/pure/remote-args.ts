// The compiler online (web/remote.ts): how the arguments of a command are
// turned into the job's. A path of the folder sent becomes `w/...`, another
// absolute path (a temporary file) `t/<n>/<name>`, the target of -o or
// --output `o/<n>/<name>`; everything else stays as it is.

const OUTPUT_FLAGS = new Set(['-o', '--output']);

export interface ArgumentPlan {
  args: string[];
  /** Job path, then the extension's path: to turn the printed text back. */
  back: [string, string][];
  /** Where the files written under each output slot go. */
  outputs: { slot: string; target: string }[];
  /** Files outside the folder that the job needs: read them, then send them at `jobPath`. */
  reads: { jobPath: string; source: string }[];
}

const dirOf = (p: string) => p.slice(0, Math.max(0, p.lastIndexOf('/'))) || '/';
const nameOf = (p: string) => p.slice(p.lastIndexOf('/') + 1);
export const insidePath = (p: string, dir: string) => p === dir || p.startsWith(`${dir}/`);

export function planArguments(args: string[], base: string): ArgumentPlan {
  const plan: ArgumentPlan = { args: [], back: [], outputs: [], reads: [] };
  let slots = 0;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    const eq = arg.startsWith('-') ? arg.indexOf('=') : -1;
    const flag = eq > 0 ? arg.slice(0, eq) : null;
    const value = flag ? arg.slice(eq + 1) : arg;
    if (!value.startsWith('/')) {
      plan.args.push(arg);
      continue;
    }
    const isOutput = OUTPUT_FLAGS.has(flag ?? '') || (!flag && i > 0 && OUTPUT_FLAGS.has(args[i - 1]));
    let jobPath: string;
    if (isOutput) {
      jobPath = `o/${++slots}/${nameOf(value)}`;
      plan.outputs.push({ slot: `o/${slots}/`, target: dirOf(value) });
    } else if (insidePath(value, base)) {
      jobPath = `w${value.slice(base.length)}` || 'w';
    } else {
      jobPath = `t/${++slots}/${nameOf(value)}`;
      plan.reads.push({ jobPath, source: value });
    }
    plan.back.push([jobPath, value]);
    plan.args.push(flag ? `${flag}=${jobPath}` : jobPath);
  }
  // Longest first, so that a path is never turned back by a shorter one it contains.
  plan.back.sort((a, b) => b[0].length - a[0].length);
  return plan;
}

/** The job's paths in a printed text turned back into the extension's. */
export function restorePaths(text: string, back: [string, string][]): string {
  return back.reduce((t, [from, to]) => t.split(from).join(to), text);
}

/** Where a file the job wrote goes, or null when it is not an output asked for. */
export function outputTarget(rel: string, outputs: { slot: string; target: string }[]): string | null {
  const out = outputs.find((o) => rel.startsWith(o.slot));
  return out ? `${out.target}/${rel.slice(out.slot.length)}` : null;
}
