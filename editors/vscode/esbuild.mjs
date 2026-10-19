// Bundles the extension into dist/extension.js and the workbench page into
// dist/webview.js, and copies the compiler's diagram viewer (the files of
// src/compiler/diagram/viewer in this repository) into media/viewer so the
// page runs them from the extension's own files. `vscode` is provided by the
// host; everything else is bundled.
import * as esbuild from 'esbuild';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const production = process.argv.includes('--production');
const watch = process.argv.includes('--watch');
const here = path.dirname(fileURLToPath(import.meta.url));

/** The viewer files the workbench page loads, in order (see src/workbench/panel.ts). */
const VIEWER_FILES = [
  'vendor/elk.bundled.js',
  'vendor/ELK-LICENSE.md',
  'arcviz-render.js',
  'arcviz-links.js',
  'arcviz-navigation.js',
  'arcviz-collapse.js',
  'arcviz-scope.js',
  'arcviz-export.js',
  'arcviz-stage.js',
  'arcviz-place.js',
  'arcviz-edit.js',
  'arcviz-viewer.js',
  'arcviz-viewer.css',
];

function copyViewer() {
  const source = process.env.ARCLANG_VIEWER_DIR ?? path.resolve(here, '..', '..', 'src', 'compiler', 'diagram', 'viewer');
  const target = path.join(here, 'media', 'viewer');
  if (!fs.existsSync(path.join(source, 'arcviz-viewer.js'))) {
    if (fs.existsSync(path.join(target, 'arcviz-viewer.js'))) {
      console.log(`viewer: kept the copy in ${path.relative(here, target)} (no source at ${source})`);
      return;
    }
    throw new Error(`the diagram viewer was not found at ${source}; set ARCLANG_VIEWER_DIR to the compiler's src/compiler/diagram/viewer`);
  }
  for (const file of VIEWER_FILES) {
    const from = path.join(source, file);
    const to = path.join(target, file);
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.copyFileSync(from, to);
  }
  console.log(`viewer: ${VIEWER_FILES.length} files copied to ${path.relative(here, target)}`);
}

copyViewer();

const common = {
  bundle: true,
  sourcemap: !production,
  minify: production,
  logLevel: 'info',
};

const host = await esbuild.context({
  ...common,
  entryPoints: ['src/extension.ts'],
  format: 'cjs',
  platform: 'node',
  target: 'node20',
  outfile: 'dist/extension.js',
  external: ['vscode'],
});

const page = await esbuild.context({
  ...common,
  entryPoints: ['webview/main.ts'],
  format: 'iife',
  platform: 'browser',
  target: 'es2022',
  outfile: 'dist/webview.js',
});

if (watch) {
  await Promise.all([host.watch(), page.watch()]);
} else {
  await Promise.all([host.rebuild(), page.rebuild()]);
  await Promise.all([host.dispose(), page.dispose()]);
}
