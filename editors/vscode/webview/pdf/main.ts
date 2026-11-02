// The PDF view: every page of the document drawn by pdf.js, drawn when it
// comes into view, at the zoom chosen (fit the width, fit the page, or a step).
import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy, type RenderTask } from 'pdfjs-dist';
import { currentPage, Fit, fitScale, stepZoom, ToPdfView } from '../../src/pure/pdf-view';

declare function acquireVsCodeApi(): { postMessage(message: unknown): void };
const vscode = acquireVsCodeApi();

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const doc = $<HTMLElement>('doc');
const status = $<HTMLElement>('status');
const pageInput = $<HTMLInputElement>('page');

interface Sheet {
  element: HTMLElement;
  width: number;
  height: number;
  drawn: number;
  task: RenderTask | null;
}

let pdf: PDFDocumentProxy | null = null;
let sheets: Sheet[] = [];
let scale = 1;
let fit: Fit = 'width';

async function startWorker(): Promise<void> {
  // A webview cannot start a worker from its resources: the worker is made from its text.
  const text = await fetch(document.body.dataset.worker ?? '').then((r) => r.text());
  GlobalWorkerOptions.workerPort = new Worker(URL.createObjectURL(new Blob([text], { type: 'text/javascript' })), { type: 'module' });
}

function say(text: string): void {
  status.textContent = text;
  status.hidden = !text;
}

const observer = new IntersectionObserver((entries) => {
  for (const entry of entries) if (entry.isIntersecting) void draw(sheets[Number((entry.target as HTMLElement).dataset.index)]);
}, { root: doc, rootMargin: '600px 0px' });

async function draw(sheet: Sheet | undefined): Promise<void> {
  if (!pdf || !sheet || sheet.drawn === scale) return;
  sheet.task?.cancel();
  const page = await pdf.getPage(Number(sheet.element.dataset.index) + 1);
  const viewport = page.getViewport({ scale });
  const ratio = window.devicePixelRatio || 1;
  const canvas = document.createElement('canvas');
  canvas.width = Math.floor(viewport.width * ratio);
  canvas.height = Math.floor(viewport.height * ratio);
  const drawnAt = scale;
  sheet.task = page.render({ canvas, viewport, transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0] });
  try {
    await sheet.task.promise;
    sheet.element.replaceChildren(canvas);
    sheet.drawn = drawnAt;
  } catch {
    /* canceled: a newer zoom draws it */
  } finally {
    sheet.task = null;
  }
}

function layout(): void {
  if (sheets.length === 0) return;
  if (fit !== 'none') scale = fitScale(fit, sheets[0], { width: doc.clientWidth, height: doc.clientHeight });
  for (const sheet of sheets) {
    sheet.element.style.width = `${Math.floor(sheet.width * scale)}px`;
    sheet.element.style.height = `${Math.floor(sheet.height * scale)}px`;
  }
  $<HTMLButtonElement>('zoom').textContent = `${Math.round(scale * 100)} %`;
  $<HTMLButtonElement>('width').setAttribute('aria-pressed', String(fit === 'width'));
  $<HTMLButtonElement>('page-fit').setAttribute('aria-pressed', String(fit === 'page'));
  for (const sheet of sheets) {
    observer.unobserve(sheet.element);
    observer.observe(sheet.element);
  }
}

/** Zooms, keeping the place read under the same point of the view. */
function zoomTo(next: Fit | number): void {
  const before = scale;
  const anchor = doc.scrollTop / Math.max(1, doc.scrollHeight);
  if (typeof next === 'number') {
    fit = 'none';
    scale = next;
  } else fit = next;
  layout();
  if (scale !== before) doc.scrollTop = anchor * doc.scrollHeight;
}

async function open(name: string, bytes: Uint8Array): Promise<void> {
  say(`Opening ${name}…`);
  const position = doc.scrollTop / Math.max(1, doc.scrollHeight);
  await pdf?.destroy();
  pdf = await getDocument({ data: bytes, useWorkerFetch: false, standardFontDataUrl: document.body.dataset.fonts }).promise;
  document.title = name;
  sheets = [];
  const elements: HTMLElement[] = [];
  for (let n = 1; n <= pdf.numPages; n++) {
    const { width, height } = (await pdf.getPage(n)).getViewport({ scale: 1 });
    const element = document.createElement('section');
    element.className = 'sheet';
    element.dataset.index = String(n - 1);
    element.setAttribute('aria-label', `Page ${n}`);
    sheets.push({ element, width, height, drawn: 0, task: null });
    elements.push(element);
  }
  doc.replaceChildren(...elements);
  $<HTMLElement>('count').textContent = `/ ${pdf.numPages}`;
  pageInput.max = String(pdf.numPages);
  layout();
  doc.scrollTop = position * doc.scrollHeight;
  say('');
}

function goTo(page: number): void {
  const sheet = sheets[Math.min(sheets.length, Math.max(1, page)) - 1];
  sheet?.element.scrollIntoView({ block: 'start' });
}

doc.addEventListener('scroll', () => {
  pageInput.value = String(currentPage(sheets.map((s) => s.element.offsetTop - doc.offsetTop), doc.scrollTop, doc.clientHeight));
}, { passive: true });
pageInput.addEventListener('change', () => goTo(Number(pageInput.value)));
$('in').addEventListener('click', () => zoomTo(stepZoom(scale, 1)));
$('out').addEventListener('click', () => zoomTo(stepZoom(scale, -1)));
$('zoom').addEventListener('click', () => zoomTo(1));
$('width').addEventListener('click', () => zoomTo('width'));
$('page-fit').addEventListener('click', () => zoomTo('page'));
$('save').addEventListener('click', () => vscode.postMessage({ type: 'outside' }));
new ResizeObserver(() => {
  if (fit !== 'none') layout();
}).observe(doc);
doc.addEventListener('wheel', (event) => {
  if (!event.ctrlKey && !event.metaKey) return;
  event.preventDefault();
  zoomTo(stepZoom(scale, event.deltaY < 0 ? 1 : -1));
}, { passive: false });
window.addEventListener('keydown', (event) => {
  if (event.target === pageInput || event.altKey) return;
  const key = event.key;
  if (key === '+' || key === '=') zoomTo(stepZoom(scale, 1));
  else if (key === '-') zoomTo(stepZoom(scale, -1));
  else if (key === '0') zoomTo(1);
  else if (key === 'w' || key === 'W') zoomTo('width');
  else if (key === 'p' || key === 'P') zoomTo('page');
  else if (key === 'Home') goTo(1);
  else if (key === 'End') goTo(sheets.length);
  else return;
  event.preventDefault();
});

window.addEventListener('message', (event: MessageEvent<ToPdfView>) => {
  const message = event.data;
  if (message.type === 'error') say(message.text);
  else if (message.type === 'pdf') void open(message.name, Uint8Array.from(atob(message.base64), (c) => c.charCodeAt(0))).catch((error: unknown) => say(`This PDF cannot be shown: ${error instanceof Error ? error.message : String(error)}`));
});

void startWorker().then(
  () => vscode.postMessage({ type: 'ready' }),
  (error: unknown) => say(`The PDF viewer cannot start: ${error instanceof Error ? error.message : String(error)}`),
);
