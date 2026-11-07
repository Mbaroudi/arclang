# ArcLang in the browser: VS Code for the Web, no server per visitor

Status: prototype on branches `web-ext` (this repository) and `web-vscode`
(the site repository). Nothing is published or deployed.

The playground at arclang.fr/playground becomes the ArcLang workbench inside
a VS Code for the Web, running our own extension. The visitor's models live in
the page (memory, kept in IndexedDB) and never reach a server of ours; the
only text that leaves the page is what is sent to an AI the visitor chooses.

```
page (static files)
 ├─ VS Code workbench           monaco-vscode-api 38 (VS Code 1.142 sources, MIT)
 ├─ extension host worker       runs the ArcLang web extension
 │    └─ WebAssembly compiler   wasm/ (the same Rust core as the CLI)
 ├─ webviews (iframes)          the workbench editor, the AI chat
 └─ virtual file system         file:///workspace, in memory, saved to IndexedDB
```

## 1. Choosing how to get a VS Code workbench as static files

| | (a) monaco-vscode-api | (b) Code - OSS web build | (c) OpenVSCode Server's web client |
|---|---|---|---|
| What it is | npm packages built from the VS Code sources, one per service; MIT | `gulp vscode-web-min` of microsoft/vscode | the browser half of Gitpod's server build |
| License | MIT (all `@codingame/monaco-vscode-*`; third-party parts MIT/BSD/ISC/MPL/Apache, see Licenses) | MIT for Code - OSS (the product name and marketplace are not) | MIT |
| Works without a server | yes: extension host in a worker, files service with our own provider | yes, with a vendor-supplied setup | no: its client talks to a remote agent (file system, extension host, terminal) over a WebSocket |
| Custom editors and webviews | yes (tested) | yes | yes, through the server |
| Extension host in a worker, virtual FS | yes (tested) | yes | not without rewriting the client |
| Size we measured | 23 MB raw, 5.7 MB gzip, 207 files (whole page, ArcLang included) | not built | server tarball 77 MB (linux-x64, v1.109.5); the client alone is not shipped |
| Effort / risk | days; npm install and Vite | a 1.6 GB repository, an 8 GB-heap gulp build, a rebuild at every VS Code release, our own packaging | a rewrite of the remote layer |

Decision: **(a)**. It is (b) already built, cut in services we choose, with the
API to register an extension, a file provider and workbench options. Tried:
`npm i` of 41 packages (140 MB of `node_modules`), a Vite build in 3 to 20 s,
the full flow in Chromium against the static files and against the site's
nginx in Docker. Not tried: (b) (the build was not run: 1.6 GB of sources, hours
of CPU, nothing it would give that (a) lacks) and (c) (the client cannot start
without its server). Their rows come from the repositories' documentation and
release assets.

Not used, by decision: the proprietary builds that `@vscode/test-web`
downloads, and Microsoft's marketplace. The extension gallery is set to Open
VSX (`productConfiguration.extensionsGallery`); it is only queried when the
visitor opens the Extensions view.

### Licenses

Everything shipped to the page is MIT except: `jschardet` (LGPL-2.1+, text
encoding detection of the workbench; kept in its own chunk, `assets/jschardet-*.js`,
so it can be replaced), `dompurify` (MPL-2.0 or Apache-2.0), the Codicon font
(icons CC-BY-4.0, attribution in the product), the Seti icon theme (MIT).
`vscode-cdn.net` appears in requests of the webviews: it is a virtual origin
that the page's own service worker answers (checked by making the host
unresolvable: everything still works); nothing is fetched from Microsoft.

## 2. The extension as a web extension

`editors/vscode/package.json`: `browser: ./dist/web/extension.js`,
`main` and `browser` together (no `extensionKind`: vsce accepts only `ui` and `workspace`, and VS Code runs an extension that has both on the desktop and on the web), `capabilities.virtualWorkspaces.supported: true`.
The desktop build (`main`, `dist/extension.js`) is unchanged.

Every compiler call of the extension goes through `src/cli.ts`, `run(args)`.
`run` now sends the call to a `Backend`: the native one (`execFile`, as before)
or the WebAssembly one (`src/web/wasm-backend.ts`), installed by the web entry
(`src/web/extension.ts`) with `setBackend`. The WebAssembly backend reads and
writes the temporary files of those calls on an in-memory file system
(`src/web/vfs.ts`); esbuild replaces `node:fs`, `node:fs/promises`, `node:os`,
`node:crypto`, `node:zlib`, `node:child_process` (a stub never reached) and
`node:path` (path-browserify) in the web bundle only, and injects Node's
`Buffer`. `vscode-languageclient`, the binary installer and the MCP provider are
not in the web bundle; diagnostics come from `check` (`src/web/diagnostics.ts`).

Honest limit: the compiler-facing path (snapshot, edits, check, format,
SysML v2) uses temporary files on the shim, not `vscode.workspace.fs`; model
files themselves (`.arc`, layout files) are read and written through
`vscode.workspace.fs` already. Moving the temp files to `vscode.workspace.fs`
would only matter for a compiler that reads several files.

### What the WebAssembly build answers

| `arclang ...` | in the browser |
|---|---|
| `workbench` (snapshot with the diagram payload), `check --json`, `edit`, `format`, `export -f sys-ml`, `import -f sysml`, `metamodel`, `rules`, `transition` (plan and step) | yes (new wasm exports: `workbench`, `check_json`, `export_sysml`, `metamodel_markdown`, `rules_markdown`; `transition` and `import_sysml` existed) |
| `gate`, `table`/tables view, `impact`, `diff`, `export` to other formats, capella import, `verify`, `diagram -f html` | "not available in the browser yet": the command says so before it asks anything (`refusedHere` in `src/cli.ts`), the backend answers exit code 78 with the same sentence; never a crash |

What is missing, with an estimate each (the compiler core is already linked
in the wasm; the work is the export and the glue):

| Item | Estimate |
|---|---|
| `gate` (production gate report) | 0.5 day: `compiler::production_gate` is in the snapshot already, only the text report is missing |
| tables (`table`, views of requirements, traces...) | 1 day: `compiler::tables`, one export per spec and a JSON in |
| `impact` and `diff` | 1 day each: both take two models; the diff needs the old text (VS Code's git is absent in the browser: compare with the saved copy) |
| exports (ReqIF, C headers, Protobuf, HTML dossier, Mermaid, PlantUML, json/yaml/xml) | 0.5 day for the ones the snapshot already renders (`exports: true`), 1 day for the rest |
| `diagram -f html` standalone page | 0.5 day: not needed, the workbench replaces it (the command opens the workbench) |
| Capella import (`.capella`, XMI) | 2 days: needs a zip/deflate in the worker, and `capella_importer` on text |
| `verify` | not feasible as is: it runs external tools (simulation, SAT); a server tier |
| PowerPoint and image exports of the workbench | 1 day: needs `zlib.deflateRawSync`, replace by `fflate` (MIT, 8 kB) |
| Models of several files (`import "..."`) | 1.5 days: the wasm needs a virtual file resolver fed by `vscode.workspace.fs` |

## 3. The page

`web/tools/vscode-web` (site repository): a Vite project booting the
workbench with `getWorkbenchServiceOverride`, a virtual workspace of three
sample models (`starter.arc`, `flight_control.arc`, `ev_product_line.arc`) and
`battery.mo`, the extension registered in the worker host, the first model
opened beside its workbench. `tools/build_vscode_web.sh` builds the compiler,
the extension, the page and puts the result in `site/playground/vscode/`
(not committed). nginx serves it with the headers of
`site/nginx-vscode-headers.conf`.

The workspace is an in-memory provider (`file:///workspace`) filled from
IndexedDB when it has files, else from the samples, and saved 400 ms after each
change. A private window that refuses IndexedDB gets a workspace for the
session.

### Measured (headless Chromium, M-series Mac, localhost, the machine busy)

| | |
|---|---|
| first visit, files | 66 files (page, workers, extension, compiler), 21 MB raw |
| first visit, gzip -6 | 5.7 MB (the workbench 2.9 MB, the compiler 0.85 MB, the extension host 0.5 MB, the layout engine 0.47 MB) |
| time to the first rendered diagram, cold | about 1.3 s on localhost (add the transfer: 5.7 MB is about 1 s at 50 Mbit/s) |
| warm reload (same tab, HTTP cache) | about 0.75 s |

`web/tools/vscode-web` is served by the site's nginx in Docker for the test
(`test/web/web_playground_test.js` runs against it too).

## 4. The AI chat

The web VS Code has no Copilot chat, so the extension carries its own: a
webview view of the secondary side bar (`arclang.chat`, `src/ai/chat/`), in
the desktop and in the web build. It reuses the layers of `src/ai/`: the
providers over `fetch`, the context builder, `runAsk`/`applySession` (the
compiler checks each edit on a copy, the answer is a diff, Apply is one
undoable edit), `SecretStorage` for the keys, the log.

- It sees the active file and the selection (and the model of the active
  workbench tab). The header shows what it sees: language, file, selected
  lines and elements.
- On `.arc`: the question goes through the engine of Ask AI. The compiler
  refuses what does not compile and sends its refusal back to the AI up to
  `arclang.ai.repairAttempts` times; the card shows the checked edits, the new
  findings and the diff, with Apply, Undo, Open diff, Refine, Reject. On a
  model that does not compile the AI answers in words and says so.
- On `.mo`: the AI explains, writes and fixes. Every answer carries
  "Not verified: there is no Modelica compiler in the browser. Checking and
  simulating the model (OpenModelica) will run in the full workspace tier,
  later." Code blocks offer Copy, Replace selection, Insert at cursor, Replace
  file (one undoable edit).
- Modelica support in the extension: language `modelica` for `*.mo`, a
  TextMate grammar and a language configuration written for this repository
  (`syntaxes/modelica.tmLanguage.json`, `modelica-language-configuration.json`;
  keywords from the language specification, no third-party grammar).
  `battery.mo` is in the sample workspace.

### Providers in a browser

`arclang-hosted` (the page's default) goes to the site's gateway; the others
go straight from the visitor's browser to the provider. CORS, tested with
preflight requests from `Origin: https://arclang.fr` (October 2026):

| Provider | From a browser | Note |
|---|---|---|
| ArcLang hosted | yes | same origin, `/api/ai`, no key in the request |
| Anthropic | yes | the request adds `anthropic-dangerous-direct-browser-access: true` (added only in a worker) |
| OpenAI | yes | `access-control-allow-origin` echoes the origin |
| Google Gemini | yes | key in `x-goog-api-key` |
| Mistral | yes | `access-control-allow-origin: *` |
| Azure OpenAI | only if the resource's CORS rules allow the site | not testable without a resource |
| Ollama (local) | only with `OLLAMA_ORIGINS=https://arclang.fr` set on the visitor's machine, and Chrome's local-network permission prompt | not tested with a real Ollama |
| OpenAI-compatible (LM Studio, vLLM...) | only if that server answers CORS preflights | the fake LLM of the tests does |
| VS Code language models | no | no Copilot in a browser; hidden from the provider list |

Keys: `ArcLang: Set AI API key` stores a key in `SecretStorage` (the browser's
secret storage: IndexedDB, not in settings, not in a workspace file; it
survives a reload, checked), never in a setting. The test checks that the key
reaches the provider only, and is in no request to another host.

CSP: the page may reach its own origin and ONE provider, the one the visitor
configured. When a key is set or a provider chosen, the page sets a
`SameSite=Strict` cookie `arclang_ai=<provider>` (path of the page);
nginx maps the cookie to one origin of a fixed list (`map $cookie_arclang_ai
$ai_connect`) and adds it to `connect-src` when it serves the page; the page
offers a reload, since a policy applies to the next load. An unknown cookie
adds nothing (checked on the container). Limit: a provider whose address is
not in the list (Azure, a custom server) cannot be reached from the hosted
page; the desktop extension has no such limit.

### The gateway of the hosted AI

`site/gateway/server.mjs` (Node 20, no dependency; `server.test.mjs`, 8 tests),
started beside nginx in the same container (`gateway/start.sh` in
`/docker-entrypoint.d`), so one Fly app serves the site and the AI. nginx
proxies `/api/ai` to it, unbuffered (SSE).

| Guardrail | How |
|---|---|
| fixed models | `claude-haiku-5-5` (default) and `claude-sonnet-5-5`; the first playground's `claude-sonnet-5` is an alias; anything else is a 400 |
| caps | `max_tokens` at most 4096 (2048 if absent); body at most 200 kB (nginx 256 kB); at most 8 tools; only `model`, `max_tokens`, `messages`, `stream`, `system`, `tools`, `tool_choice`, `temperature`, `stop_sequences` go upstream |
| topic | the request must mention ArcLang, Arcadia, Modelica, `.arc`, `.mo` or have their shapes (equations, `parameter Real`, `end X;`, `component "..."`); the chat's own system prompts always do; else 422 |
| rate | nginx `limit_req` 10 requests a minute, burst 5, per client (`Fly-Client-IP`), and the same token bucket in the gateway; a friendly JSON 429 |
| spend | a daily counter, UTC day, kept in memory and in a file; a worst-case reservation before each request (input bytes as tokens, the whole output allowance), settled with the real usage read from the stream; at `AI_DAILY_BUDGET_USD` (default 25) every request is a 429 `kind: budget` with a friendly message that points to the visitor's own key; prices per model in `AI_PRICES` (defaults are assumptions: 1/5 and 3/15 USD per million tokens) |
| privacy | nothing of a prompt or an answer is logged or stored; the log line is status, model, token counts, duration, day; the upstream's error text is not forwarded |
| key | `ANTHROPIC_API_KEY` from the environment (`fly secrets set`); never in a file; empty: the gateway says the shared AI is off |

Limits: the counter lives in the machine's `/tmp` (a new machine starts the
day at zero; with `min_machines_running = 0` and auto-stop, set a volume or
accept it, or put the counter in Redis/Upstash when there is more than one
machine); the per-client buckets are per machine too.

## 5. Tests

All in `editors/vscode/test/web/`, Chromium through Playwright:

- `web_playground_test.js`: load, extension active in the worker, the sample
  in the workbench with its diagram, a palette tool edits the text, undo,
  `check` on a broken model, the workspace survives a reload, no request to
  another host, no CSP violation. Run against Vite's output and against the
  site's nginx image.
- `web_ai_test.js`: with the fake LLM (`test/ai/fake-llm-server.js`, now with
  CORS) as an OpenAI-compatible provider: a key typed in the workbench's
  input, a selection in the `.arc`, a compiler-checked proposal and its diff,
  Apply, the keyboard undo (typed as a French keyboard sends it, `azerty.js`:
  the page follows the visitor's real layout), a `.mo` file with the
  not-verified label, the key only at the provider.
- `web_hosted_test.js`: the hosted path on the Docker image with the gateway
  and a fake Anthropic behind it.
- `timing.js`: cold and warm load.
- Unit: `test/unit/chat.test.ts` (Modelica prompt, code blocks, the page's
  message checks, the hosted provider); the gateway's `server.test.mjs`.

## 6. Risks

- `'unsafe-eval'` in the CSP of `/playground/vscode/`: the workbench compiles a
  few functions at run time. The rest of the site keeps its strict policy.
- The webviews share the page's origin (arclang.fr has no wildcard sub-domain):
  an extension's webview could reach the page. Acceptable while the only
  extension is ours; with the Open VSX gallery open, a malicious extension is
  the visitor's own choice, but the isolation is weaker than a desktop's. A
  wildcard `*.arclang.fr` and `iframeAlternateDomain` give each webview its own origin.
- Size: 5.7 MB gzip on the first visit. Most of it is the workbench; services
  the playground does not use can still be removed.
- The extension host runs in a worker: a long compile (the 450 KB In-Flight
  Entertainment model) blocks that worker, not the page.
- monaco-vscode-api follows VS Code monthly: the version is pinned; each update
  needs the test suite re-run.
- The secret storage of a browser is not a keychain: the key sits in the
  visitor's IndexedDB, readable by any script of the origin. The CSP and the
  absence of third-party scripts are what protect it; say so in the UI.
- The Anthropic prices in the gateway are assumptions: set `AI_PRICES`.
- The hosted AI is public: the daily cap bounds the loss, not the abuse; watch
  the one log line per request.
- The French (or any non-US) keyboard: VS Code reads the real layout; the
  tests send the keys as that layout does.

## 7. Deploy

1. `tools/build_vscode_web.sh` (needs the arclang checkout beside: wasm-pack,
   binaryen, node 20); `site/playground/vscode/` is not committed.
2. `fly secrets set ANTHROPIC_API_KEY=...`, check `AI_DAILY_BUDGET_USD` in
   `fly.toml`, `fly deploy` from `site/`. The image is nginx plus Node
   (`apk add nodejs`, about 50 MB more).
3. Check `/api/ai/status` (`enabled: true`), then the page.
4. Cloudflare or another CDN in front: cache `/playground/vscode/assets/*`
   (immutable) and let `/api/ai` through unbuffered.
