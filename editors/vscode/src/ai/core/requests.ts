// The edit requests an AI may propose: checked for shape here, then by the
// compiler. Anything else in an answer is ignored, never run.
import type { EditRequest } from '../../pure/workbench';

const STRING = 'string';
type Field = [name: string, required: boolean];

const SHAPES: Record<string, Field[]> = {
  set: [['element', true], ['key', true], ['value', true]],
  unset: [['element', true], ['key', true]],
  rename: [['element', true], ['name', true]],
  add: [['keyword', true], ['name', true], ['container', false], ['layer', false], ['layer_name', false]],
  add_raw: [['declaration', true], ['container', false], ['layer', false], ['layer_name', false]],
  remove: [['element', true]],
  move: [['element', true], ['container', false], ['layer', false], ['layer_name', false]],
  add_trace: [['from', true], ['kind', true], ['to', true]],
  remove_trace: [['from', true], ['kind', true], ['to', true]],
  reconnect: [['kind', true], ['from', true], ['to', true], ['name', false], ['new_from', false], ['new_to', false]],
  duplicate: [['element', true], ['container', false], ['layer', false], ['layer_name', false]],
  wrap_fragment: [['scenario', true], ['kind', true], ['first', true], ['last', true], ['label', false]],
};

export const EDIT_OPS = Object.keys(SHAPES);
export const MAX_REQUESTS = 40;

/** The requests, each checked for its op and its fields; the first bad one is refused with its reason. */
export function validateRequests(value: unknown): { ok: true; requests: EditRequest[] } | { ok: false; error: string } {
  if (!Array.isArray(value)) return { ok: false, error: 'the edits are not a list' };
  if (value.length > MAX_REQUESTS) return { ok: false, error: `${value.length} edits: at most ${MAX_REQUESTS} in one proposal` };
  const out: EditRequest[] = [];
  for (const [index, raw] of value.entries()) {
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return { ok: false, error: `edit ${index + 1} is not an object` };
    const request = raw as Record<string, unknown>;
    const shape = typeof request.op === 'string' ? SHAPES[request.op] : undefined;
    if (!shape) return { ok: false, error: `edit ${index + 1}: unknown op ${JSON.stringify(request.op)} (one of ${EDIT_OPS.join(', ')})` };
    const clean: Record<string, unknown> = { op: request.op };
    for (const [name, required] of shape) {
      const field = request[name];
      if (field === undefined || field === null || field === '') {
        if (required) return { ok: false, error: `edit ${index + 1} (${request.op}) has no ${name}` };
        continue;
      }
      // A value may come as a number or a boolean: it is written as the model writes it.
      if (typeof field !== STRING && !(name === 'value' && (typeof field === 'number' || typeof field === 'boolean'))) return { ok: false, error: `edit ${index + 1} (${request.op}): ${name} is not text` };
      clean[name] = String(field);
    }
    if (request.op === 'add' && Array.isArray(request.attributes)) {
      const pairs = (request.attributes as unknown[]).filter((p): p is [unknown, unknown] => Array.isArray(p) && p.length === 2).map(([k, v]) => [String(k), String(v)] as [string, string]);
      if (pairs.length) clean.attributes = pairs;
    }
    if ((request.op === 'add' || request.op === 'add_raw') && request.top_level === true) clean.top_level = true;
    if (request.op === 'wrap_fragment' && Array.isArray(request.operands)) {
      clean.operands = (request.operands as unknown[]).filter((p): p is [unknown, unknown] => Array.isArray(p) && p.length === 2).map(([a, b]) => [String(a), String(b)]);
    }
    out.push(clean as EditRequest);
  }
  return { ok: true, requests: out };
}

const where = (r: { container?: string; layer?: string; top_level?: boolean }) => (r.container ? `inside ${r.container}` : r.layer ? `in ${r.layer}` : 'at the top level');

/** One request in words. */
export function describeEdit(request: EditRequest): string {
  switch (request.op) {
    case 'set':
      return `set ${request.key}: ${request.value} on ${request.element}`;
    case 'unset':
      return `remove ${request.key} from ${request.element}`;
    case 'rename':
      return `rename ${request.element} to "${request.name}"`;
    case 'add':
      return `add ${request.keyword} "${request.name}" ${where(request)}`;
    case 'add_raw':
      return `add ${request.declaration.split('\n')[0].replace(/\s*\{\s*$/, '')} ${where(request)}`;
    case 'remove':
      return `delete ${request.element}`;
    case 'move':
      return `move ${request.element} ${where(request)}`;
    case 'add_trace':
      return `trace ${request.from} ${request.kind} ${request.to}`;
    case 'remove_trace':
      return `remove the trace ${request.from} ${request.kind} ${request.to}`;
    case 'reconnect':
      return `reconnect the ${request.kind} ${request.from} → ${request.to}${request.new_from ? ` from ${request.new_from}` : ''}${request.new_to ? ` to ${request.new_to}` : ''}`;
    case 'duplicate':
      return `duplicate ${request.element}`;
    case 'wrap_fragment':
      return `wrap ${request.first}…${request.last} of ${request.scenario} in a ${request.kind} fragment`;
    default:
      return JSON.stringify(request);
  }
}

/** The JSON Schema of one request, for the tools the AI may call. */
export const EDIT_REQUEST_SCHEMA = {
  type: 'object',
  description:
    'One `arclang edit` request. op is one of: set {element,key,value}, unset {element,key}, rename {element,name}, add {keyword,name,attributes?:[[key,literal]],container?|layer?}, add_raw {declaration,container?|layer?}, remove {element}, move {element,container?|layer?}, add_trace {from,kind,to}, remove_trace {from,kind,to}, reconnect {kind,from,to,name?,new_from?,new_to?}, duplicate {element}. Values are ArcLang literals: strings in double quotes ("ASIL-B"), quantities bare (20 ms). layer is the block keyword: operational_analysis, system_analysis, logical_architecture, physical_architecture, epbs.',
  properties: {
    op: { type: 'string', enum: EDIT_OPS },
    element: { type: 'string' },
    key: { type: 'string' },
    value: { type: 'string' },
    name: { type: 'string' },
    keyword: { type: 'string' },
    attributes: { type: 'array', items: { type: 'array', items: { type: 'string' } } },
    declaration: { type: 'string' },
    container: { type: 'string' },
    layer: { type: 'string' },
    from: { type: 'string' },
    to: { type: 'string' },
    kind: { type: 'string' },
    new_from: { type: 'string' },
    new_to: { type: 'string' },
  },
  required: ['op'],
};
