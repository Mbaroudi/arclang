# Vendored layout engine

`elk.bundled.js` is [elkjs](https://github.com/kieler/elkjs) 0.9.3, copied
unmodified from the npm package (`elkjs/lib/elk.bundled.js`). It is the
Eclipse Layout Kernel compiled to JavaScript.

- License: Eclipse Public License 2.0 — see `ELK-LICENSE.md`. ArcLang itself
  is MIT; this file stays under the EPL and is distributed unmodified.
- Source: https://github.com/kieler/elkjs (tag `0.9.3`).
- Used by: the diagram viewer (embedded in exported HTML so the document
  works offline) and `tools/diagram_render/render.mjs` (headless rendering).

To upgrade, replace the file with the same path from a newer npm release,
update the version and the integrity hash in `src/compiler/diagram/html.rs`,
then run `tools/diagram_render/check.sh`.
