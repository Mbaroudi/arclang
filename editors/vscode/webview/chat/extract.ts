// The text of a file the engineer attaches to the chat (requirements,
// specifications, other models): plain text as it is, XML and ReqIF without
// their markup, Word (.docx) and ReqIF archives (.reqifz) unzipped, PDF read
// by pdf.js. Loaded by the chat page only when a file is attached.
import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist';

const TEXT = /\.(txt|md|markdown|csv|tsv|json|ya?ml|arc|sysml|kerml|mo|mos|ini|toml)$/i;
const MARKUP = /\.(xml|reqif|html?|xhtml|svg)$/i;

import { decode, markupText, unzip } from './markup';

async function pdfText(bytes: Uint8Array, workerUrl: string): Promise<string> {
  if (!GlobalWorkerOptions.workerPort) {
    // A webview cannot start a worker from its resources: the worker is made from its text.
    const text = await fetch(workerUrl).then((r) => r.text());
    GlobalWorkerOptions.workerPort = new Worker(URL.createObjectURL(new Blob([text], { type: 'text/javascript' })), { type: 'module' });
  }
  const task = getDocument({ data: bytes });
  const pdf = await task.promise;
  const pages: string[] = [];
  for (let n = 1; n <= pdf.numPages; n++) {
    const content = await (await pdf.getPage(n)).getTextContent();
    let line = '';
    const lines: string[] = [];
    for (const item of content.items) {
      if (!('str' in item)) continue;
      line += item.str;
      if (item.hasEOL) {
        lines.push(line);
        line = '';
      }
    }
    if (line) lines.push(line);
    pages.push(lines.join('\n'));
  }
  await task.destroy();
  return pages.join('\n\n');
}

/** The text of an attached file; an error names what cannot be read. */
export async function extractText(name: string, bytes: Uint8Array, workerUrl: string): Promise<string> {
  if (TEXT.test(name)) return decode(bytes);
  if (MARKUP.test(name)) return markupText(decode(bytes));
  if (/\.docx$/i.test(name)) {
    const files = await unzip(bytes, (n) => n === 'word/document.xml');
    const xml = files.get('word/document.xml');
    if (!xml) throw new Error('a Word file without its document');
    return markupText(decode(xml));
  }
  if (/\.reqifz$/i.test(name)) {
    const files = await unzip(bytes, (n) => /\.reqif$/i.test(n));
    return [...files.values()].map((b) => markupText(decode(b))).join('\n\n');
  }
  if (/\.pdf$/i.test(name)) return pdfText(bytes, workerUrl);
  throw new Error('only text, Markdown, CSV, JSON, XML, ReqIF, Word (.docx), PDF and model files (.arc, .sysml, .mo) can be attached');
}
