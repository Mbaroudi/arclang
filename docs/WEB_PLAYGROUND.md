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
| Licence | MIT (all `@codingame/monaco-vscode-*`; third-party parts MIT/BSD/ISC/MPL/Apache, see Licences) | MIT for Code - OSS (the product name and marketplace are not) | MIT |
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

### Licences

Everything shipped to the page is MIT except: `jschardet` (LGPL-2.1+, text
encoding detection of the workbench; kept in its own chunk, `assets/jschardet-*.js`,
so it can be replaced), `dompurify` (MPL-2.0 or Apache-2.0), the Codicon font
(icons CC-BY-4.0, attribution in the product), the Seti icon theme (MIT).
`vscode-cdn.net` appears in requests of the webviews: it is a virtual origin
that the page's own service worker answers (checked by making the host
unresolvable: everything still works); nothing is fetched from Microsoft.

## 2. The extension as a web extension

`editors/vscode/package.json`: `browser: ./dist/web/extension.js`,
`extensionKind: [workspace, web]`, `capabilities.virtualWorkspaces.supported: true`.
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
| `workbench` (snapshot with the diagram payload), `check --json`, `edit`, `format`, `export -f sys-ml`, `import -f sysml`, `metamodel`, `rules` | yes (new wasm exports: `workbench`, `check_json`, `export_sysml`, `metamodel_markdown`, `rules_markdown`) |
| `gate`, `table`/tables view, `impact`, `diff`, `transition` (command), `export` to other formats, capella import, `verify`, `diagram -f html` | "not available in the browser yet" (a clear message, exit code 2, never a crash) |

What is missing, with an estimate each (the compiler core is already linked
in the wasm; the work is the export and the glue):

| Item | Estimate |
|---|---|
| `gate` (production gate report) | 0.5 day: `compiler::production_gate` is in the snapshot already, only the text report is missing |
| tables (`table`, views of requirements, traces...) | 1 day: `compiler::tables`, one export per spec and a JSON in |
| `impact` and `diff` | 1 day each: both take two models; the diff needs the old text (VS Code's git is absent in the browser: compare with the saved copy) |
| `transition` command | 0.5 day: the export exists (`transition`), only the arguments need mapping |
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

## 4. Risks

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
