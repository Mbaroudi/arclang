// AI design assistant — bring-your-own Anthropic API key.
// The key is stored in localStorage and requests go DIRECTLY from the
// browser to api.anthropic.com: consistent with the playground's privacy
// promise — neither the model nor the key ever touches our server.

const API_URL = 'https://api.anthropic.com/v1/messages';
const MODEL = 'claude-sonnet-5';
const KEY_STORAGE = 'arclang_anthropic_key';

const SYSTEM_PROMPT = `You are a senior systems engineer designing models in ArcLang, \
a strict text language for the Arcadia MBSE method. You ALWAYS answer with a short \
explanation followed by ONE complete .arc model in a single \`\`\`arc code fence — \
the full model, never a fragment, since it replaces the editor content.

ArcLang syntax reference (exhaustive — use ONLY these constructs):
model Name { version: "1.0" }
requirements safety { req "REQ-001" "Title" { description: "..." safety_level: "ASIL-B" priority: "High" } }
architecture logical { component "Name" { id: "LC-001" port in PortName { } port out Other { } function "Do thing" { latency: "25 ms" } } component_exchange "name" { from_port: "LC-001" to_port: "LC-002" } }
architecture physical { node "ECU" { id: "PC-001" } link "bus" { from: "PC-001" to: "PC-002" protocol: "CAN" } deployment { deploy "LC-001" on "PC-001" } }
operational_analysis "Name" { actor "Driver" { id: "OA-ACT-001" } }
trace "LC-001" satisfies "REQ-001" { rationale: "..." }   (also: refines, realizes, implements, validates)
test_case "TC-001" { name: "Bench test" verifies: ["REQ-001"] method: "test" }   (methods: test|analysis|inspection|demonstration)
safety_analysis { hazard "HAZ-001" { description: "..." severity: "S3" exposure: "E4" controllability: "C1" asil: "ASIL-B" mitigated_by: ["REQ-001"] } }
state_machine Name { initial: "Off" mode Off { } mode Active { } transition Off -> Active { trigger: "engage" } }
mission "MIS-001" { name: "..." }  capability "CAP-001" { name: "..." realizes: ["OC-001"] involves: ["SF-001"] }
functional_chain "FC-001" { name: "..." steps: ["SF-001", "SF-002"] latency_budget: "100 ms" }

Hard rules: every id quoted; every trace endpoint must reference a declared id; \
if any element declares an ASIL, include a safety_analysis with a hazard whose \
S/E/C is consistent with that ASIL (ISO 26262 table 4); every requirement should \
be satisfied by a trace and verified by a test_case so the production gate passes. \
When the user reports compiler errors, fix them minimally without redesigning.`;

const $ = (id) => document.getElementById(id);
let history = [];

function getKey() { return localStorage.getItem(KEY_STORAGE) || ''; }

function addMsg(role, text) {
  const div = document.createElement('div');
  div.className = 'ai-msg ' + role;
  div.textContent = text;
  $('ai-log').appendChild(div);
  $('ai-log').scrollTop = $('ai-log').scrollHeight;
  return div;
}

function extractArc(text) {
  const match = text.match(/```(?:arc|arclang)?\n([\s\S]*?)```/);
  return match ? match[1] : null;
}

async function send() {
  const input = $('ai-input');
  const question = input.value.trim();
  if (!question) return;
  const key = getKey();
  if (!key) { $('ai-settings').open = true; $('ai-key').focus(); return; }

  input.value = '';
  addMsg('user', question);
  const pending = addMsg('assistant', '…');

  const pg = window.arclangPG;
  const result = pg.getLastResult();
  const diagnostics = result && !result.success
    ? '\n\nCurrent compiler errors:\n' + result.errors.join('\n')
    : (result && result.warnings.length ? '\n\nCompiler warnings:\n' + result.warnings.join('\n') : '');

  history.push({
    role: 'user',
    content: `${question}\n\nCurrent editor content:\n\`\`\`arc\n${pg.getSource()}\n\`\`\`${diagnostics}`,
  });

  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'x-api-key': key,
        'anthropic-version': '2023-06-01',
        'anthropic-dangerous-direct-browser-access': 'true',
      },
      body: JSON.stringify({
        model: MODEL,
        max_tokens: 8000,
        system: SYSTEM_PROMPT,
        messages: history.slice(-6),
      }),
    });
    if (!response.ok) {
      const err = await response.json().catch(() => ({}));
      throw new Error(err?.error?.message || `API ${response.status}`);
    }
    const data = await response.json();
    const text = data.content.map((b) => b.text || '').join('');
    history.push({ role: 'assistant', content: text });

    const arc = extractArc(text);
    pending.textContent = text.replace(/```(?:arc|arclang)?\n[\s\S]*?```/, '').trim()
      || 'Model updated.';
    if (arc) {
      const btn = document.createElement('button');
      btn.className = 'ai-apply';
      btn.textContent = '→ Apply to editor & compile';
      btn.onclick = () => { pg.setSource(arc.trim() + '\n'); btn.disabled = true; btn.textContent = '✓ Applied'; };
      pending.appendChild(btn);
      // Auto-apply: the promise of the assistant is design-in-the-loop.
      btn.click();
    }
  } catch (e) {
    pending.textContent = '✗ ' + e.message;
    pending.classList.add('error');
    history.pop();
  }
}

function init() {
  $('ai-toggle').addEventListener('click', () => {
    document.body.classList.toggle('ai-open');
    if (document.body.classList.contains('ai-open') && !getKey()) $('ai-settings').open = true;
  });
  $('ai-key').value = getKey();
  $('ai-key').addEventListener('change', (e) => localStorage.setItem(KEY_STORAGE, e.target.value.trim()));
  $('ai-send').addEventListener('click', send);
  $('ai-input').addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }
  });
}

init();
