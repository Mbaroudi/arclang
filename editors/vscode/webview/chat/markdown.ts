// The prose of an answer as the reader expects it: paragraphs, headings,
// numbered and bulleted lists (also when the AI wrote "1. … 2. …" in one
// line), **bold**, *italic*, `code`, links and the documentation's addresses.
// Built node by node from text: nothing the AI writes becomes markup.

import { inlineSteps } from './steps';

type Make = (tag: string, attrs?: Record<string, string>, text?: string) => HTMLElement;

const make: Make = (tag, attrs = {}, text) => {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, v);
  if (text !== undefined) node.textContent = text;
  return node;
};

/** Inline marks of a line: `code`, **bold**, *italic*, [text](url), bare links. */
export function inline(host: HTMLElement, text: string, onLink: (url: string) => void): void {
  const pattern = /(`[^`\n]+`)|(\*\*[^*\n]+\*\*)|(\*[^*\s][^*\n]*\*)|(\[[^\]\n]+\]\((?:https?:\/\/|arclang\.fr)[^)\s]+\))|((?:https?:\/\/|\b)arclang\.fr\/[\w\-./#]*[\w/])|(https?:\/\/[^\s)<>"]+[\w/])/g;
  let last = 0;
  for (const m of text.matchAll(pattern)) {
    if (m.index! > last) host.append(text.slice(last, m.index));
    const token = m[0];
    if (m[1]) host.append(make('code', {}, token.slice(1, -1)));
    else if (m[2]) {
      const strong = make('strong');
      inline(strong, token.slice(2, -2), onLink);
      host.append(strong);
    } else if (m[3]) host.append(make('em', {}, token.slice(1, -1)));
    else {
      const label = m[4] ? token.slice(1, token.indexOf('](')) : token;
      const raw = m[4] ? token.slice(token.indexOf('](') + 2, -1) : token;
      const url = /^https?:\/\//.test(raw) ? raw : `https://${raw}`;
      const a = make('a', { href: url, title: url }, label);
      a.addEventListener('click', (event) => {
        event.preventDefault();
        onLink(url);
      });
      host.append(a);
    }
    last = m.index! + token.length;
  }
  if (last < text.length) host.append(text.slice(last));
}

/** A block of prose (no code fence) into paragraphs, headings and lists. */
export function prose(host: HTMLElement, text: string, onLink: (url: string) => void): void {
  for (const block of text.split(/\n{2,}/)) {
    const lines = block.split('\n').map((l) => l.trimEnd()).filter((l) => l.trim());
    if (!lines.length) continue;
    const heading = /^(#{1,4})\s+(.*)$/.exec(lines[0]);
    if (heading && lines.length === 1) {
      const h = make(heading[1].length <= 2 ? 'h3' : 'h4');
      inline(h, heading[2], onLink);
      host.append(h);
      continue;
    }
    if (lines.every((l) => /^\s*[-*•]\s+/.test(l))) {
      const ul = make('ul');
      for (const l of lines) {
        const li = make('li');
        inline(li, l.replace(/^\s*[-*•]\s+/, ''), onLink);
        ul.append(li);
      }
      host.append(ul);
      continue;
    }
    if (lines.every((l) => /^\s*\d{1,2}[.)]\s+/.test(l))) {
      const ol = make('ol');
      for (const l of lines) {
        const li = make('li');
        inline(li, l.replace(/^\s*\d{1,2}[.)]\s+/, ''), onLink);
        ol.append(li);
      }
      host.append(ol);
      continue;
    }
    const joined = lines.join(' ');
    const steps = inlineSteps(joined);
    if (steps) {
      if (steps.lead) {
        const p = make('p');
        inline(p, steps.lead, onLink);
        host.append(p);
      }
      const ol = make('ol');
      for (const item of steps.items) {
        const li = make('li');
        inline(li, item, onLink);
        ol.append(li);
      }
      host.append(ol);
      continue;
    }
    // A note of the assistant's own work ("(the compiler refused it: …; asking for a correction)") reads as a status line.
    if (/^\((the compiler|correcting|asking)/.test(joined) || /^\(the compiler .*\)$/.test(joined)) {
      host.append(make('p', { class: 'chat-work' }, `⟳ ${joined.replace(/^\(|\)$/g, '')}`));
      continue;
    }
    const p = make('p');
    lines.forEach((l, i) => {
      if (i) p.append(make('br'));
      inline(p, l, onLink);
    });
    host.append(p);
  }
}
