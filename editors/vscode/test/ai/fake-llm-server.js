// A local stand-in for the AI providers: it speaks the streaming wire format
// of OpenAI (and Azure, Mistral, OpenAI-compatible servers), Anthropic,
// Google Gemini and Ollama, and answers from a script. No network, no key,
// no cost. Used by the unit tests (in process) and the real VS Code test
// (as a child process: `node fake-llm-server.js [port] [scenario]`, prints
// `LISTEN <url>`).
'use strict';
const http = require('node:http');

/** What a request is: its wire format, from its path. */
function formatOf(url) {
  if (/\/chat\/completions$/.test(url.split('?')[0])) return 'openai';
  if (/\/v1\/messages$/.test(url)) return 'anthropic';
  if (/:streamGenerateContent/.test(url)) return 'google';
  if (/\/api\/chat$/.test(url)) return 'ollama';
  return null;
}

/** The text of the last user turn, in any of the four formats. */
function lastUser(format, body) {
  const text = (content) => (typeof content === 'string' ? content : Array.isArray(content) ? content.map((c) => c.text || (typeof c.content === 'string' ? c.content : '') || '').join('') : '');
  if (format === 'google') {
    const turns = (body.contents || []).filter((c) => c.role === 'user');
    const last = turns[turns.length - 1];
    return last ? last.parts.map((p) => p.text || (p.functionResponse ? JSON.stringify(p.functionResponse.response) : '')).join('') : '';
  }
  const turns = (body.messages || []).filter((m) => m.role === 'user' || m.role === 'tool');
  const last = turns[turns.length - 1];
  return last ? text(last.content) : '';
}

/** Every text the request carries, system included: what a scenario matches on. */
function allText(body) {
  return JSON.stringify(body);
}

/** Text cut in small pieces, so the clients reassemble across chunks. */
function pieces(text, size = 7) {
  const out = [];
  for (let i = 0; i < text.length; i += size) out.push(text.slice(i, i + size));
  return out;
}

function sse(res, events, delay) {
  res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
  let i = 0;
  const next = () => {
    if (res.destroyed) return;
    if (i >= events.length) return res.end();
    const e = events[i++];
    res.write(e);
    if (delay) setTimeout(next, delay);
    else setImmediate(next);
  };
  next();
}

function openaiEvents(reply) {
  const out = [];
  const chunk = (delta, finish = null) => `data: ${JSON.stringify({ id: 'c1', object: 'chat.completion.chunk', choices: [{ index: 0, delta, finish_reason: finish }] })}\n\n`;
  out.push(chunk({ role: 'assistant', content: '' }));
  for (const p of pieces(reply.text || '')) out.push(chunk({ content: p }));
  (reply.toolCalls || []).forEach((call, index) => {
    const args = JSON.stringify(call.input);
    out.push(chunk({ tool_calls: [{ index, id: `call_${index}_x`, type: 'function', function: { name: call.name, arguments: '' } }] }));
    for (const p of pieces(args, 11)) out.push(chunk({ tool_calls: [{ index, function: { arguments: p } }] }));
  });
  out.push(chunk({}, reply.toolCalls && reply.toolCalls.length ? 'tool_calls' : 'stop'));
  out.push('data: [DONE]\n\n');
  return out;
}

function anthropicEvents(reply) {
  const ev = (type, data) => `event: ${type}\ndata: ${JSON.stringify({ type, ...data })}\n\n`;
  const out = [ev('message_start', { message: { id: 'm1', role: 'assistant', usage: { input_tokens: 10 } } }), ': keep-alive comment\n\n', ev('ping', {})];
  let index = 0;
  if (reply.text) {
    out.push(ev('content_block_start', { index, content_block: { type: 'text', text: '' } }));
    for (const p of pieces(reply.text)) out.push(ev('content_block_delta', { index, delta: { type: 'text_delta', text: p } }));
    out.push(ev('content_block_stop', { index }));
    index++;
  }
  for (const call of reply.toolCalls || []) {
    out.push(ev('content_block_start', { index, content_block: { type: 'tool_use', id: `toolu_${index}`, name: call.name, input: {} } }));
    for (const p of pieces(JSON.stringify(call.input), 9)) out.push(ev('content_block_delta', { index, delta: { type: 'input_json_delta', partial_json: p } }));
    out.push(ev('content_block_stop', { index }));
    index++;
  }
  out.push(ev('message_delta', { delta: { stop_reason: reply.toolCalls && reply.toolCalls.length ? 'tool_use' : 'end_turn' }, usage: { output_tokens: 5 } }));
  out.push(ev('message_stop', {}));
  // Lines ended by \r\n, as some proxies send them.
  return out.map((e) => e.replace(/\n/g, '\r\n'));
}

function googleEvents(reply) {
  const out = [];
  for (const p of pieces(reply.text || '', 12)) out.push(`data: ${JSON.stringify({ candidates: [{ content: { role: 'model', parts: [{ text: p }] } }] })}\n\n`);
  for (const call of reply.toolCalls || []) out.push(`data: ${JSON.stringify({ candidates: [{ content: { role: 'model', parts: [{ functionCall: { name: call.name, args: call.input } }] } }] })}\n\n`);
  out.push(`data: ${JSON.stringify({ candidates: [{ content: { role: 'model', parts: [] }, finishReason: 'STOP' }], usageMetadata: { promptTokenCount: 3, candidatesTokenCount: 4 } })}\n\n`);
  return out;
}

function ollamaLines(reply) {
  const out = [];
  for (const p of pieces(reply.text || '')) out.push(JSON.stringify({ model: 'm', message: { role: 'assistant', content: p }, done: false }) + '\n');
  if (reply.toolCalls && reply.toolCalls.length) out.push(JSON.stringify({ model: 'm', message: { role: 'assistant', content: '', tool_calls: reply.toolCalls.map((c) => ({ function: { name: c.name, arguments: c.input } })) }, done: false }) + '\n');
  out.push(JSON.stringify({ model: 'm', message: { role: 'assistant', content: '' }, done: true, done_reason: 'stop', prompt_eval_count: 3, eval_count: 4 }) + '\n');
  return out;
}

/**
 * Starts the server. `script(request)` returns the reply: `{ text, toolCalls:
 * [{name, input}], status, errorBody, delayMs (before answering), chunkDelayMs,
 * hang (never answer), redirectTo (a 302 there), flood: {bytes, newlines} }`.
 */
function startFakeServer(script, port = 0) {
  const requests = [];
  const server = http.createServer((req, res) => {
    // Cross-origin by design: the browser tests call it from the page of another origin.
    res.setHeader('access-control-allow-origin', '*');
    res.setHeader('access-control-allow-headers', '*');
    res.setHeader('access-control-allow-methods', 'GET, POST, OPTIONS');
    if (req.method === 'OPTIONS') {
      res.writeHead(204);
      return res.end();
    }
    let raw = '';
    req.on('data', (c) => (raw += c));
    req.on('end', () => {
      const format = formatOf(req.url);
      let body = {};
      try { body = raw ? JSON.parse(raw) : {}; } catch { body = { _invalid: raw }; }
      if (req.method === 'GET' && req.url === '/__requests') {
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(requests));
      }
      const request = { format, url: req.url, headers: req.headers, body, lastUser: format ? lastUser(format, body) : '', all: allText(body) };
      requests.push(request);
      if (!format) { res.writeHead(404); return res.end('{"error":{"message":"no such endpoint"}}'); }
      const reply = script(request) || { text: 'OK' };
      const answer = () => {
        if (reply.hang) return; // the client's timeout or cancellation ends it
        if (reply.redirectTo) {
          res.writeHead(reply.redirectStatus || 302, { location: reply.redirectTo });
          return res.end();
        }
        if (reply.flood) {
          // A server gone wrong: `bytes` of text, one endless line unless `newlines`.
          res.writeHead(200, { 'content-type': 'text/event-stream' });
          const piece = (reply.flood.newlines ? 'data: {"x":1}\n\n' : 'x').repeat(64 * 1024 / (reply.flood.newlines ? 16 : 1));
          let sent = 0;
          const next = () => {
            if (res.destroyed) return;
            if (sent >= reply.flood.bytes) return res.end();
            sent += piece.length;
            if (res.write(piece)) setImmediate(next);
            else res.once('drain', next);
          };
          return next();
        }
        if (reply.status && reply.status >= 400) {
          res.writeHead(reply.status, { 'content-type': 'application/json' });
          return res.end(reply.errorBody || JSON.stringify({ error: { message: 'scripted failure' } }));
        }
        if (format === 'ollama') {
          res.writeHead(200, { 'content-type': 'application/x-ndjson' });
          const lines = ollamaLines(reply);
          let i = 0;
          const next = () => { if (res.destroyed) return; if (i >= lines.length) return res.end(); res.write(lines[i++]); if (reply.chunkDelayMs) setTimeout(next, reply.chunkDelayMs); else setImmediate(next); };
          return next();
        }
        const events = format === 'openai' ? openaiEvents(reply) : format === 'anthropic' ? anthropicEvents(reply) : googleEvents(reply);
        sse(res, events, reply.chunkDelayMs || 0);
      };
      if (reply.delayMs) setTimeout(answer, reply.delayMs);
      else answer();
    });
  });
  return new Promise((resolve) => {
    server.listen(port, '127.0.0.1', () => {
      const address = server.address();
      resolve({ url: `http://127.0.0.1:${address.port}`, requests, close: () => new Promise((r) => { server.closeAllConnections?.(); server.close(() => r()); }) });
    });
  });
}

// --- The scenario of the real VS Code test ------------------------------------

/** Answers as a modeling assistant would, keyed on what is asked. */
function arclangScenario(request) {
  const asked = request.lastUser;
  const all = request.all;
  if (/Reply with the single word OK/.test(asked)) return { text: 'OK' };
  // The impact suggestions: a trace was removed, propose to put one back.
  if (/propose_changes/.test(all) && /trace\.component|has no trace|satisfied by no/.test(all)) {
    return {
      text: '',
      toolCalls: [{
        name: 'propose_changes',
        input: {
          proposals: [
            { title: 'Trace the camera to the latency requirement again', rationale: 'LC-CAM delivers the lane model the 80 ms budget of REQ-LKA-001 depends on; without its satisfies trace the requirement loses one of its two contributors.', resolves: 'Component LC-CAM has no trace', requests: [{ op: 'add_trace', from: 'LC-CAM', kind: 'satisfies', to: 'REQ-LKA-001' }] },
          ],
        },
      }],
    };
  }
  if (/monitoring component/i.test(asked)) {
    return {
      text: 'I add a Health Monitor in the logical architecture, with an input port per watched component.',
      toolCalls: [{
        name: 'propose_edits',
        input: {
          explanation: 'A Health Monitor component watches the Camera and the LKA Controller: one input port for each, an output port on each of them and a component exchange from each to the monitor.',
          requests: [
            { op: 'add', keyword: 'component', name: 'Health Monitor', attributes: [['id', '"LC-MON"']], layer: 'logical_architecture' },
            { op: 'add_raw', declaration: 'port in "cam_health"', container: 'LC-MON' },
            { op: 'add_raw', declaration: 'port in "ctl_health"', container: 'LC-MON' },
            { op: 'add_raw', declaration: 'port out "health"', container: 'LC-CAM' },
            { op: 'add_raw', declaration: 'port out "health"', container: 'LC-CTL' },
            { op: 'add_raw', declaration: 'component_exchange "CameraHealth" {\n  from_port: "LC-CAM.health"\n  to_port: "LC-MON.cam_health"\n}', layer: 'logical_architecture' },
            { op: 'add_raw', declaration: 'component_exchange "ControllerHealth" {\n  from_port: "LC-CTL.health"\n  to_port: "LC-MON.ctl_health"\n}', layer: 'logical_architecture' },
          ],
        },
      }],
    };
  }
  if (/ASIL-D|safety level/i.test(asked)) {
    return { text: 'Raising the safety level of the selected component.\n```json\n{"explanation": "The camera feeds an ASIL-B chain; it is set to ASIL-D as asked.", "requests": [{"op": "set", "element": "LC-CAM", "key": "safety_level", "value": "\\"ASIL-D\\""}]}\n```' };
  }
  if (/explain/i.test(asked)) return { text: 'The selection is the camera: it captures frames and sends the lane model to the controller.' };
  return { text: 'No change is needed.' };
}

if (require.main === module) {
  const port = Number(process.argv[2] || 0);
  startFakeServer(arclangScenario, port).then((s) => console.log(`LISTEN ${s.url}`));
}

module.exports = { startFakeServer, arclangScenario, formatOf };
