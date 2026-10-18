// Pure helpers of the extension: no dependency on the `vscode` module, so
// they are unit-tested with node:test.

/** The last ```arc (or ```arclang) fenced block of a chat answer, or null. */
export function extractArcFence(text: string): string | null {
  const fence = /```(?:arc|arclang)\s*\n([\s\S]*?)```/g;
  let last: string | null = null;
  for (const match of text.matchAll(fence)) last = match[1];
  if (last !== null) return last.replace(/\s+$/, '') + '\n';
  // A whole answer that is a model without a fence.
  if (/^\s*model\b/.test(text) && /\{/.test(text)) return text.trim() + '\n';
  return null;
}

/** The import format the CLI expects for a file name, or null when the user must choose. */
export function importFormatFor(fileName: string): string | null {
  const lower = fileName.toLowerCase();
  const byExtension: Array<[RegExp, string]> = [
    [/\.sysml$/, 'sysml'],
    [/\.(capella|aird|melodymodeller|afm)$/, 'capella'],
    [/\.reqif(z)?$/, 'req-if'],
    [/\.(mmd|mermaid)$/, 'mermaid'],
    [/\.(puml|plantuml|pu)$/, 'plant-uml'],
    [/\.(yaml|yml)$/, 'yaml'],
  ];
  for (const [pattern, format] of byExtension) if (pattern.test(lower)) return format;
  return null;
}

export const IMPORT_FORMATS = ['sysml', 'capella', 'req-if', 'doors', 'mermaid', 'plant-uml', 'json', 'yaml', 'xml'] as const;

/** Export formats of `arclang export -f`, with a label and the extension of the file written. */
export const EXPORT_FORMATS: ReadonlyArray<{ format: string; label: string; extension: string }> = [
  { format: 'sys-ml', label: 'SysML v2 textual notation', extension: 'sysml' },
  { format: 'sysml-json', label: 'SysML v2 API records (abstract syntax)', extension: 'json' },
  { format: 'capella', label: 'Capella project', extension: 'capella' },
  { format: 'req-if', label: 'ReqIF requirements interchange', extension: 'reqif' },
  { format: 'c-headers', label: 'C interface headers', extension: 'h' },
  { format: 'proto', label: 'Protocol Buffers interface', extension: 'proto' },
  { format: 'markdown', label: 'Markdown report', extension: 'md' },
  { format: 'html', label: 'HTML report', extension: 'html' },
  { format: 'pdf', label: 'PDF report', extension: 'pdf' },
  { format: 'mermaid', label: 'Mermaid diagrams', extension: 'mmd' },
  { format: 'plant-uml', label: 'PlantUML diagrams', extension: 'puml' },
  { format: 'json', label: 'ArcLang JSON', extension: 'json' },
  { format: 'yaml', label: 'ArcLang YAML', extension: 'yaml' },
  { format: 'xml', label: 'ArcLang XML', extension: 'xml' },
  { format: 'simulink', label: 'Simulink skeleton', extension: 'm' },
  { format: 'fmi', label: 'FMI model description', extension: 'xml' },
  { format: 'terraform', label: 'Terraform (deployment)', extension: 'tf' },
];

export interface Finding {
  rule: string;
  severity: string;
  message: string;
  element?: string | null;
  fix?: unknown;
}

export interface CheckReport {
  file?: string;
  errors: string[];
  findings: Finding[];
}

/** `arclang check --json` output, read leniently: errors may be strings or objects. */
export function parseCheckReport(stdout: string): CheckReport | null {
  let raw: unknown;
  try {
    raw = JSON.parse(stdout);
  } catch {
    return null;
  }
  if (!raw || typeof raw !== 'object') return null;
  const record = raw as Record<string, unknown>;
  const errors = Array.isArray(record.errors)
    ? record.errors.map((e) => (typeof e === 'string' ? e : JSON.stringify(e)))
    : [];
  const findings = Array.isArray(record.findings) ? (record.findings as Finding[]) : [];
  return { file: typeof record.file === 'string' ? record.file : undefined, errors, findings };
}

/** One line per error and per finding, grouped by severity, for a chat answer or an output channel. */
export function summarizeCheck(report: CheckReport): string {
  const lines: string[] = [];
  if (report.errors.length > 0) {
    lines.push(`${report.errors.length} compiler error${report.errors.length > 1 ? 's' : ''}:`);
    for (const error of report.errors) lines.push(`  - ${error}`);
  }
  const bySeverity = new Map<string, Finding[]>();
  for (const finding of report.findings) {
    const list = bySeverity.get(finding.severity) ?? [];
    list.push(finding);
    bySeverity.set(finding.severity, list);
  }
  for (const severity of ['error', 'warning', 'info']) {
    const list = bySeverity.get(severity);
    if (!list) continue;
    lines.push(`${list.length} ${severity}${list.length > 1 ? 's' : ''}:`);
    for (const finding of list) lines.push(`  - [${finding.rule}] ${finding.message}`);
  }
  if (lines.length === 0) lines.push('The model compiles with no finding.');
  return lines.join('\n');
}

export interface TransitionCandidate {
  id: string;
  name: string;
  kind: string;
  realized_by: string[];
}

export interface TransitionPlan {
  step: string;
  label: string;
  candidates: TransitionCandidate[];
  blocked?: string | null;
}

/** The transition plan of `arclang transition --plan` as readable text. */
export function renderTransitionPlan(plans: TransitionPlan[], file: string): string {
  const lines: string[] = [];
  for (const plan of plans) {
    const pending = plan.candidates.filter((c) => c.realized_by.length === 0);
    const realized = plan.candidates.length - pending.length;
    lines.push(`${plan.label} (${plan.step}): ${realized} realized, ${pending.length} to create${plan.blocked ? `; ${plan.blocked}` : ''}`);
    for (const candidate of plan.candidates) {
      const mark = candidate.realized_by.length ? 'done' : 'todo';
      const by = candidate.realized_by.length ? ` realized by ${candidate.realized_by.join(', ')}` : '';
      lines.push(`  ${mark}  ${candidate.kind} ${candidate.name}${by}`);
    }
    if (pending.length > 0) lines.push(`  next: arclang transition ${file} --step ${plan.step} --write`);
    lines.push('');
  }
  return lines.join('\n').trimEnd() + '\n';
}

interface MetamodelType {
  type: string;
  of?: MetamodelType | string;
  name?: string;
  dimension?: string;
  [key: string]: unknown;
}

interface MetamodelAttribute {
  key: string;
  type: MetamodelType;
  doc?: string;
}

interface MetamodelKind {
  name: string;
  keyword?: string;
  keywords?: string[];
  layer?: string;
  arcadia?: string;
  doc?: string;
  attributes?: MetamodelAttribute[];
}

interface Metamodel {
  version?: string;
  kinds?: MetamodelKind[];
  enums?: Array<{ name: string; values: string[]; doc?: string }>;
  trace_rules?: Array<{ kind: string; from: string; to: string; doc?: string }>;
  units?: Array<{ symbol: string; dimension: string }>;
}

/** The ArcLang keyword(s) that declare each metaclass (as the language server maps them). */
const KEYWORDS: Record<string, string> = {
  Actor: 'actor', SystemActor: 'actor', OperationalEntity: 'entity', OperationalCapability: 'operational_capability',
  OperationalActivity: 'activity', OperationalProcess: 'operational_process', OperationalExchange: 'interaction',
  Requirement: 'requirement (or req "ID" "Title")', Mission: 'mission', Capability: 'capability', CapabilityRealization: 'capability_realization',
  SystemFunction: 'function', LogicalFunction: 'function', PhysicalFunction: 'function', FunctionPort: 'port in|out "name"',
  FunctionalExchange: 'functional_exchange', SystemComponent: 'component', LogicalComponent: 'component',
  BehaviorComponent: 'behavior_component', HardwareComponent: 'hardware_component', FunctionalChain: 'functional_chain',
  ControlNode: 'control', SequenceLink: 'sequence', ComponentPort: 'port in|out "name"', PhysicalPort: 'port in|out "name"',
  LogicalInterface: 'interface', ComponentExchange: 'component_exchange', PhysicalNode: 'node', PhysicalLink: 'physical_link (or link)',
  PhysicalExchange: 'physical_exchange', Deployment: 'deployment A -> B (or deploys "A" inside a node)', PhysicalPath: 'physical_path',
  EpbsSystem: 'configuration_item', EpbsSubsystem: 'configuration_item', EpbsItem: 'configuration_item',
  Class: 'class', Association: 'association', Enumeration: 'enumeration', DataType: 'data_type', ExchangeItem: 'exchange_item',
  Hazard: 'hazard', FmeaEntry: 'fmea', Threat: 'threat', PrimaryAsset: 'asset', Trace: 'trace "A" <kind> "B"', TestCase: 'test_case',
  Model: 'model', Type: 'type', Constraint: 'constraint', StateMachine: 'state_machine', State: 'state (or mode)', Transition: 'transition A -> B',
  Diagram: 'diagram', Scenario: 'scenario', Fragment: 'fragment', StateFragment: 'state (inside a scenario lifeline)', Message: 'message "A" -> "B" "text"',
};

function typeText(type: MetamodelType | string | undefined): string {
  if (!type) return '?';
  if (typeof type === 'string') return type;
  const inner = type.of ? `<${typeText(type.of)}>` : type.name ? `<${type.name}>` : type.dimension ? `<${type.dimension}>` : '';
  return `${type.type.toLowerCase()}${inner}`;
}

/**
 * The metamodel JSON of `arclang metamodel --format json`, condensed to what a
 * language model needs to write ArcLang: kinds per layer with their typed
 * attributes, enumerations, trace rules and units. Roughly a fifth of the
 * Markdown the compiler prints.
 */
export function condenseMetamodel(json: string): string {
  const metamodel = JSON.parse(json) as Metamodel;
  const lines: string[] = [`ArcLang metamodel ${metamodel.version ?? ''} (printed by \`arclang metamodel\`).`];
  const byLayer = new Map<string, MetamodelKind[]>();
  for (const kind of metamodel.kinds ?? []) {
    const layer = kind.layer ?? 'Transverse';
    const list = byLayer.get(layer) ?? [];
    list.push(kind);
    byLayer.set(layer, list);
  }
  for (const [layer, kinds] of byLayer) {
    lines.push('', `## ${layer}`);
    for (const kind of kinds) {
      const keyword = kind.keyword ?? (kind.keywords && kind.keywords.join('|')) ?? KEYWORDS[kind.name];
      const head = keyword ? `${kind.name} (keyword \`${keyword}\`)` : kind.name;
      const attributes = (kind.attributes ?? [])
        .filter((a) => a.key !== 'id' && a.key !== 'name' && a.key !== 'description')
        .map((a) => `${a.key}: ${typeText(a.type)}`)
        .join(', ');
      lines.push(`- ${head}: ${kind.doc ?? ''}${attributes ? ` Attributes: id, name, description, ${attributes}.` : ' Attributes: id, name, description.'}`);
    }
  }
  if (metamodel.enums?.length) {
    lines.push('', '## Enumerations');
    for (const e of metamodel.enums) lines.push(`- ${e.name}: ${e.values.join(', ')}`);
  }
  if (metamodel.trace_rules?.length) {
    lines.push('', '## Trace rules (`trace "A" <kind> "B"`)');
    for (const rule of metamodel.trace_rules) lines.push(`- ${rule.kind}: from ${rule.from} to ${rule.to}. ${rule.doc ?? ''}`);
  }
  if (metamodel.units?.length) {
    const byDimension = new Map<string, string[]>();
    for (const unit of metamodel.units) {
      const list = byDimension.get(unit.dimension) ?? [];
      list.push(unit.symbol);
      byDimension.set(unit.dimension, list);
    }
    lines.push('', '## Units of quantities (written unquoted: `latency: 20 ms`)');
    for (const [dimension, symbols] of byDimension) lines.push(`- ${dimension}: ${symbols.join(' ')}`);
  }
  return lines.join('\n') + '\n';
}

/** Splits a command line into command and arguments, quotes respected. */
export function splitCommand(line: string): string[] {
  const parts: string[] = [];
  for (const match of line.matchAll(/"([^"]*)"|'([^']*)'|(\S+)/g)) parts.push(match[1] ?? match[2] ?? match[3]);
  return parts;
}

/** The first line of a tool answer: what the exit code says. */
export function exitLine(command: string, code: number): string {
  if (code === 0) return `\`arclang ${command}\` succeeded (exit 0).`;
  if (code === 127) return `\`arclang\` was not found: install the compiler or set the setting arclang.binary.`;
  return `\`arclang ${command}\` failed (exit ${code}).`;
}

/** Cuts a long text for a prompt or a tool result, saying how much was left out. */
export function clip(text: string, max: number): string {
  if (text.length <= max) return text;
  return `${text.slice(0, max)}\n... (${text.length - max} more characters not shown)`;
}
