// A real host for the workbench page: edits go through `arclang edit`, the
// snapshot through `arclang workbench`. Serves the page from the harness folder.
const http = require('http'); const fs = require('fs'); const path = require('path'); const os = require('os');
const { execFileSync } = require('child_process');
const BIN = process.env.ARCLANG || path.resolve(__dirname, '../../../../target/release/arclang');
const ROOT = process.argv[2]; const MODEL = process.argv[3];
let source = fs.readFileSync(MODEL, 'utf8');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'wb-e2e-'));
const file = path.join(dir, 'model.arc');
function snapshot() { fs.writeFileSync(file, source); return execFileSync(BIN, ['workbench', file], { maxBuffer: 1 << 28 }).toString(); }
function edit(requests) {
  let text = source;
  for (const [i, r] of requests.entries()) {
    fs.writeFileSync(file, text);
    let out;
    try { out = execFileSync(BIN, ['edit', file, '--request', JSON.stringify(r)], { maxBuffer: 1 << 26 }).toString(); }
    catch (e) { out = (e.stdout || '').toString() || (e.stderr || '').toString(); }
    let answer; try { answer = JSON.parse(out); } catch { answer = { success: false, error: out }; }
    if (!answer.success) return { ok: false, error: answer.error, at: i };
    text = answer.source;
  }
  source = text; return { ok: true };
}
http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/edit') {
    let body = ''; req.on('data', (c) => (body += c)); req.on('end', () => {
      const { requests } = JSON.parse(body); const result = edit(requests);
      res.writeHead(200, { 'content-type': 'application/json' }); res.end(JSON.stringify(result));
    }); return;
  }
  if (req.url.startsWith('/snapshot')) { res.writeHead(200, { 'content-type': 'application/json' }); res.end(snapshot()); return; }
  if (req.url.startsWith('/source')) { res.writeHead(200, { 'content-type': 'text/plain' }); res.end(source); return; }
  if (req.url.startsWith('/reset')) { source = fs.readFileSync(MODEL, 'utf8'); res.end('ok'); return; }
  const f = path.join(ROOT, decodeURIComponent(req.url.split('?')[0]));
  fs.readFile(f, (err, data) => { if (err) { res.writeHead(404); res.end(); return; }
    const type = f.endsWith('.js') ? 'text/javascript' : f.endsWith('.css') ? 'text/css' : f.endsWith('.json') ? 'application/json' : 'text/html';
    res.writeHead(200, { 'content-type': type }); res.end(data); });
}).listen(Number(process.env.PORT || 8766), '127.0.0.1', () => console.log('e2e host on', process.env.PORT || 8766));
