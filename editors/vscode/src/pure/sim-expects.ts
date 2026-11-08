// What an AI-written simulation must get right for its verdict to be a
// verdict and not an error: every expects: of a new test case reads a
// variable the simulated Modelica declares, by its path in the result
// (plant.water.T), with a bound in the variable's unit. Checked on the text,
// before the engineer sees the proposal: OpenModelica would only say it after
// the run ("'x' is not a variable of the simulation result").

/** One expectation of a test case: "REQ-1: max(plant.water.T) <= 96 °C". */
interface Expectation {
  text: string;
  path: string;
  /** The unit written after the bound ('' for a plain number). */
  unit: string;
}

const COMPARISON = /(<=|>=|==|!=|<|>)/;

/** The expects: strings of a model text, unescaped. */
export function expectsOf(text: string): string[] {
  const out: string[] = [];
  for (const block of text.matchAll(/expects:\s*\[([\s\S]*?)\]\s*(?:\n|\})/g)) {
    for (const item of block[1].matchAll(/"((?:[^"\\]|\\.)*)"/g)) out.push(item[1].replace(/\\"/g, '"'));
  }
  return out;
}

function parse(expect: string): Expectation | null {
  const body = expect.replace(/^\s*[\w.-]+\s*:\s*/, '');
  const [left, , right] = body.split(COMPARISON);
  if (left === undefined || right === undefined) return null;
  const inner = left.match(/\w+\(\s*([^)]*?)\s*\)/);
  const path = (inner ? inner[1] : left).trim();
  const bound = right.trim().match(/^-?[\d.]+(?:[eE][+-]?\d+)?\s*(.*)$/);
  if (!path || !bound) return null;
  return { text: expect, path, unit: bound[1].trim() };
}

/** The names the Modelica text declares, with the unit each one carries (or ''). */
function declarations(modelica: string): Map<string, string> {
  const names = new Map<string, string>();
  const code = modelica.replace(/\/\/[^\n]*|\/\*[\s\S]*?\*\//g, '');
  for (const m of code.matchAll(/(?:^|[;\n])\s*(?:(?:parameter|constant|discrete|input|output|flow|stream|replaceable|inner|outer)\s+)*([A-Z][\w.]*|Real|Integer|Boolean)\s+(\w+)\s*(\([^;]*?\))?\s*(?:=[^;]*)?(?:"[^"]*")?\s*;/g)) {
    const unit = m[3]?.match(/unit\s*=\s*"([^"]*)"/)?.[1] ?? '';
    if (!names.has(m[2]) || unit) names.set(m[2], unit);
  }
  return names;
}

/**
 * What is wrong with the expectations a proposal adds, read against the
 * Modelica it simulates: an empty list when each one can be judged.
 */
export function unreadableExpects(current: string, proposed: string, modelica: string): string[] {
  if (!modelica.trim()) return [];
  const before = new Set(expectsOf(current));
  const declared = declarations(modelica);
  const extendsGenerated = /\bextends\s+(?!Modelica\.)\w/.test(modelica);
  const problems: string[] = [];
  for (const text of expectsOf(proposed)) {
    if (before.has(text)) continue;
    const e = parse(text);
    if (!e) continue;
    if (e.path.startsWith('"')) {
      if (!extendsGenerated) problems.push(`"${text}" reads ${e.path}, an element of the model: the result only has the variables of your Modelica model. Name the variable by its path there (instance.sub.variable).`);
      continue;
    }
    const segments = e.path.split('.');
    const missing = segments.filter((s) => !declared.has(s) && s !== 'time');
    if (missing.length) {
      problems.push(`"${text}" reads ${e.path}, but ${missing.join(', ')} ${missing.length > 1 ? 'are' : 'is'} declared nowhere in the Modelica package.`);
      continue;
    }
    const unit = declared.get(segments[segments.length - 1]) ?? '';
    if (unit && !e.unit) problems.push(`"${text}" compares ${e.path} (unit "${unit}") with a plain number: write the bound with its unit.`);
    if (!unit && e.unit && segments[segments.length - 1] !== 'time') problems.push(`"${text}" gives the bound a unit (${e.unit}) but ${e.path} has none in the Modelica: declare it with unit = "…" or compare with a plain number.`);
  }
  return problems;
}
