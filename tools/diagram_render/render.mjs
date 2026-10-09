#!/usr/bin/env node
// Headless rendering of ArcLang viewpoint diagrams.
//
//   arclang diagram model.arc -f elk -o graphs.json
//   node tools/diagram_render/render.mjs graphs.json out/     # one SVG per view
//   node tools/diagram_render/render.mjs graphs.json --check  # verify only
//
// Uses the exact renderer and layout engine embedded in the HTML viewer, so
// what this draws is what the browser draws. Needs Node 18+, no npm install.
// Exits non-zero when a layout fails or an element is missing from a drawing.

import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
const ELK = require(join(viewer, 'vendor/elk.bundled.js'));
const { renderSvg, isSequence, drawnIds, esc } = require(join(viewer, 'arcviz-render.js'));

const [input, target] = process.argv.slice(2);
if (!input || !target) {
  console.error('usage: render.mjs <graphs.json> <output-dir | --check>');
  process.exit(2);
}

const graphs = JSON.parse(readFileSync(input, 'utf8'));
const elk = new ELK();
let failures = 0;

for (const graph of graphs) {
  let laidOut;
  try {
    // Scenarios are drawn as declared; everything else goes through ELK.
    laidOut = isSequence(graph) ? graph : await elk.layout(graph);
  } catch (error) {
    console.error(`✗ ${graph.id}: layout failed — ${error.message}`);
    failures += 1;
    continue;
  }
  const { svg, width, height } = renderSvg(laidOut);

  // Every node, port and edge of the graph must appear in the drawing, and
  // every edge must have been routed.
  const missing = drawnIds(laidOut).filter((id) => !svg.includes(`data-id="${esc(id)}"`));
  const unrouted = isSequence(graph)
    ? []
    : (laidOut.edges || []).filter((edge) => !(edge.sections || []).length).map((edge) => edge.id);
  for (const id of missing) console.error(`✗ ${graph.id}: '${id}' is not drawn`);
  for (const id of unrouted) console.error(`✗ ${graph.id}: edge '${id}' has no route`);
  failures += missing.length + unrouted.length;

  if (target !== '--check') {
    mkdirSync(target, { recursive: true });
    writeFileSync(join(target, `${graph.id.replace(/[^A-Za-z0-9_-]+/g, '_')}.svg`), svg);
  }
  console.log(`✓ ${graph.id}: ${width}×${height}`);
}

process.exit(failures ? 1 : 0);
