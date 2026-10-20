// Reading what the AI answered: an explanation and edit requests, from a
// tool call or a JSON block, or a whole model in an ```arc block. The answer
// is data: only these shapes are taken from it.
import type { EditRequest } from '../../pure/workbench';
import type { ToolCall } from '../providers/types';
import { validateRequests } from './requests';

export interface ParsedAnswer {
  explanation: string;
  requests: EditRequest[];
  /** A whole model text the AI wrote instead of requests. */
  source: string | null;
  /** Why the edits of the answer were not taken, when they were not. */
  problem: string | null;
}

/** The first ```lang fence of the text (```json, ```arc). */
export function fence(text: string, languages: string[]): string | null {
  const pattern = new RegExp('```(?:' + languages.join('|') + ')[ \\t]*\\r?\\n([\\s\\S]*?)```', 'i');
  const match = pattern.exec(text);
  return match ? match[1] : null;
}

/** The first JSON object of the text: a ```json fence, or the text itself. */
export function jsonObjectOf(text: string): Record<string, unknown> | null {
  const candidates = [fence(text, ['json', 'JSON', '']), text.trim()];
  for (const candidate of candidates) {
    if (!candidate) continue;
    const start = candidate.indexOf('{');
    const end = candidate.lastIndexOf('}');
    if (start < 0 || end <= start) continue;
    try {
      const value = JSON.parse(candidate.slice(start, end + 1)) as unknown;
      if (value && typeof value === 'object' && !Array.isArray(value)) return value as Record<string, unknown>;
    } catch {
      // not this one
    }
  }
  return null;
}

/** The prose of an answer without its fences. */
export function prose(text: string): string {
  return text.replace(/```[\s\S]*?```/g, '').replace(/\n{3,}/g, '\n\n').trim();
}

/** The explanation and the requests of an answer (a propose_edits call wins over the text). */
export function parseAssistAnswer(text: string, calls: ToolCall[]): ParsedAnswer {
  const call = calls.find((c) => c.name === 'propose_edits');
  const object = call ? call.input : jsonObjectOf(text);
  const explanationOf = (value: unknown) => (typeof value === 'string' ? value.trim() : '');
  if (object && (Array.isArray(object.requests) || Array.isArray(object.edits))) {
    const checked = validateRequests(object.requests ?? object.edits);
    const explanation = explanationOf(object.explanation) || prose(text);
    if (!checked.ok) return { explanation, requests: [], source: null, problem: checked.error };
    return { explanation, requests: checked.requests, source: null, problem: null };
  }
  const model = fence(text, ['arc', 'arclang']);
  if (model && /\b(model|logical_architecture|system_analysis|physical_architecture|operational_analysis|requirements)\b/.test(model)) {
    return { explanation: prose(text), requests: [], source: model, problem: null };
  }
  return { explanation: (object && explanationOf(object.explanation)) || prose(text) || text.trim(), requests: [], source: null, problem: null };
}

export interface ParsedSuggestion {
  title: string;
  rationale: string;
  resolves: string;
  requests: EditRequest[];
}

/** One to three proposals from a propose_changes call or a JSON block; bad ones dropped. */
export function parseSuggestions(text: string, calls: ToolCall[]): { suggestions: ParsedSuggestion[]; dropped: string[] } {
  const call = calls.find((c) => c.name === 'propose_changes');
  const object = call ? call.input : jsonObjectOf(text);
  const list = object && Array.isArray(object.proposals) ? object.proposals : [];
  const suggestions: ParsedSuggestion[] = [];
  const dropped: string[] = [];
  for (const raw of list.slice(0, 3)) {
    if (!raw || typeof raw !== 'object') continue;
    const value = raw as Record<string, unknown>;
    const checked = validateRequests(value.requests ?? value.edits);
    const title = typeof value.title === 'string' ? value.title.trim().slice(0, 160) : 'Proposal';
    if (!checked.ok || checked.requests.length === 0) {
      dropped.push(`${title}: ${checked.ok ? 'no edit' : checked.error}`);
      continue;
    }
    suggestions.push({
      title,
      rationale: typeof value.rationale === 'string' ? value.rationale.trim().slice(0, 1200) : '',
      resolves: typeof value.resolves === 'string' ? value.resolves.trim().slice(0, 400) : '',
      requests: checked.requests,
    });
  }
  return { suggestions, dropped };
}
