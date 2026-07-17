// ArcLang playground — the real compiler, in this tab.
// All compilation is client-side WebAssembly; no source ever leaves the page.

import init, { compile, version } from './pkg/arclang_wasm.js';

const $ = (id) => document.getElementById(id);
const src = $('src');
const gutter = $('gutter');
const status = $('status');
const DEBOUNCE_MS = 600;

const EXAMPLE_FILES = {
  starter: 'starter.arc',
  aeb: 'emergency_braking.arc',
  fcs: 'flight_control.arc',
  mission: 'mission_computer.arc',
};
const EXAMPLES = {};

async function loadExamples() {
  await Promise.all(Object.entries(EXAMPLE_FILES).map(async ([key, file]) => {
    EXAMPLES[key] = await fetch('/playground/examples/' + file).then((r) => r.text());
  }));
}

function setStatus(text, cls) {
  status.textContent = text;
  status.className = 'pg-status' + (cls ? ' ' + cls : '');
}

function syncGutter() {
  const lines = src.value.split('\n').length;
  gutter.textContent = Array.from({ length: lines }, (_, i) => i + 1).join('\n');
  gutter.scrollTop = src.scrollTop;
}

function selectTab(name) {
  document.querySelectorAll('.pg-tabs [role="tab"]').forEach((tab) => {
    tab.setAttribute('aria-selected', String(tab.dataset.tab === name));
  });
  document.querySelectorAll('.pane').forEach((pane) => {
    pane.classList.toggle('active', pane.dataset.pane === name);
  });
}

function renderDiagnostics(result) {
  const list = $('diagnostics');
  list.innerHTML = '';
  const items = [];
  for (const e of result.errors || []) items.push({ cls: 'error', text: e });
  for (const w of result.warnings || []) items.push({ cls: 'warning', text: w });
  if (!items.length) {
    items.push({ cls: 'ok', text: result.success
      ? '✓ Clean compile — no errors, no warnings. The traceability graph is fully resolved.'
      : 'No diagnostics.' });
  }
  for (const item of items) {
    const li = document.createElement('li');
    li.className = item.cls;
    li.textContent = item.text;
    list.appendChild(li);
  }
  const count = (result.errors || []).length + (result.warnings || []).length;
  const badge = $('diag-count');
  badge.hidden = count === 0;
  badge.textContent = count;
}

function renderGate(result) {
  const el = $('gate');
  if (!result.gate) {
    el.innerHTML = '<p class="pane-empty">Fix compilation errors to run the gate.</p>';
    return;
  }
  const g = result.gate;
  const verdict = g.passed
    ? `<div class="gate-verdict pass">✓ GATE PASS — ${g.requirements_verified}/${g.requirements_total} requirements verified (ISO 26262)</div>`
    : `<div class="gate-verdict fail">✗ GATE FAIL — ${g.requirements_verified}/${g.requirements_total} requirements verified (ISO 26262)</div>`;
  let rows = '';
  for (const f of g.findings) {
    rows += `<tr><td><span class="sev ${f.severity}">${f.severity}</span></td><td>${escapeHtml(f.check)}</td><td>${escapeHtml(f.message)}</td></tr>`;
  }
  el.innerHTML = verdict + (rows
    ? `<table><thead><tr><th>Severity</th><th>Check</th><th>Finding</th></tr></thead><tbody>${rows}</tbody></table>`
    : '<p>No findings. Requirements satisfied and verified, HARA consistent, timing budgets demonstrated.</p>');
}

function escapeHtml(s) {
  return s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
}

function renderExplorer(result) {
  const frame = $('explorer');
  const empty = $('explorer-empty');
  if (result.explorer_html) {
    frame.srcdoc = result.explorer_html;
    empty.hidden = true;
  } else {
    frame.removeAttribute('srcdoc');
    empty.hidden = false;
    empty.textContent = result.success
      ? 'No renderable architecture in this model yet.'
      : 'Fix compilation errors to render the explorer.';
  }
}

let timer = null;
let lastResult = null;
function scheduleCompile() {
  clearTimeout(timer);
  timer = setTimeout(runCompile, DEBOUNCE_MS);
}

function runCompile() {
  const started = performance.now();
  let result;
  try {
    result = JSON.parse(compile(src.value));
  } catch (e) {
    setStatus('compiler panic — please report this model as a bug', 'err');
    return;
  }
  lastResult = result;
  const ms = Math.round(performance.now() - started);
  renderDiagnostics(result);
  renderGate(result);
  renderExplorer(result);
  if (result.success) {
    const s = result.stats;
    setStatus(`✓ ${ms} ms — ${s.requirements} req · ${s.components} comp · ${s.functions} func · ${s.traces} traces`, 'ok');
  } else {
    setStatus(`✗ ${(result.errors[0] || 'compile error').split('\n')[0].slice(0, 90)}`, 'err');
    selectTab('diagnostics');
  }
}

async function main() {
  await Promise.all([init(), loadExamples()]);
  setStatus(`compiler v${version()} ready — wasm, client-side`);

  const params = new URLSearchParams(location.search);
  const requested = params.get('example');
  const initial = EXAMPLES[requested] ? requested : 'starter';
  $('example').value = initial;
  src.value = EXAMPLES[initial];
  syncGutter();
  runCompile();

  $('example').addEventListener('change', (e) => {
    src.value = EXAMPLES[e.target.value];
    syncGutter();
    runCompile();
  });
  $('compile').addEventListener('click', runCompile);
  src.addEventListener('input', () => { syncGutter(); scheduleCompile(); });
  src.addEventListener('scroll', () => { gutter.scrollTop = src.scrollTop; });
  src.addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); runCompile(); }
    if (e.key === 'Tab') {
      e.preventDefault();
      const { selectionStart: a, selectionEnd: b } = src;
      src.setRangeText('  ', a, b, 'end');
      syncGutter(); scheduleCompile();
    }
  });
  document.querySelectorAll('.pg-tabs [role="tab"]').forEach((tab) => {
    tab.addEventListener('click', () => selectTab(tab.dataset.tab));
  });
}

// Small API surface for the AI design assistant (ai.js).
window.arclangPG = {
  getSource: () => src.value,
  setSource: (text) => { src.value = text; syncGutter(); runCompile(); },
  compile: () => { runCompile(); return lastResult; },
  getLastResult: () => lastResult,
};

main().catch((e) => setStatus('failed to load compiler: ' + e.message, 'err'));
