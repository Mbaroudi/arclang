import assert from 'node:assert/strict';
import { test } from 'node:test';
import * as zlib from 'node:zlib';
import {
  capellaBeside,
  checkApiBase,
  countTotals,
  declarationsAfterModel,
  droppedModelFiles,
  folderImports,
  looksLikeApiRecords,
  newestFirst,
  nextPage,
  parseImportOutput,
  recordLabel,
  reqifEntryOf,
  resolveFile,
  selectionBase,
  sysmlSelection,
  withPageSize,
} from '../../src/pure/importing';
import { unzip, zip } from '../../src/pure/zip';

test('a file is imported by its extension, in the format the compiler reads', () => {
  assert.deepEqual(resolveFile('/m/Drone.capella'), { format: 'capella', input: '/m/Drone.capella', base: 'Drone' });
  assert.equal((resolveFile('/m/x.melodymodeller') as { format: string }).format, 'capella');
  assert.deepEqual(resolveFile('/m/v.sysml'), { format: 'sysml', input: '/m/v.sysml', base: 'v' });
  assert.equal((resolveFile('/m/lib.KERML') as { format: string }).format, 'sysml');
  assert.deepEqual(resolveFile('/m/reqs.reqif'), { format: 'req-if', input: '/m/reqs.reqif', base: 'reqs' });
  const archive = resolveFile('/m/reqs.reqifz') as { format: string; unzip?: boolean };
  assert.equal(archive.format, 'req-if');
  assert.equal(archive.unzip, true);
  assert.equal((resolveFile('/m/flow.mmd') as { format: string }).format, 'mermaid');
  assert.match((resolveFile('/m/notes.txt') as { error: string }).error, /imports Capella/);
});

test('an .aird is read through the .capella beside it', () => {
  const same = resolveFile('/p/Model Test 7.0.aird', ['Model Test 7.0.aird', 'Model Test 7.0.capella', 'Model Test 7.0.afm', '.project']);
  assert.deepEqual({ format: (same as { format: string }).format, input: (same as { input: string }).input }, { format: 'capella', input: '/p/Model Test 7.0.capella' });
  assert.match((same as { note: string }).note, /read through Model Test 7\.0\.capella/);
  assert.equal((capellaBeside('/p/views.aird', ['views.aird', 'Plant.capella']) as { input: string }).input, '/p/Plant.capella');
  assert.match((capellaBeside('/p/views.aird', ['views.aird']) as { error: string }).error, /no \.capella file/);
  assert.match((capellaBeside('/p/views.aird', ['A.capella', 'B.capella']) as { error: string }).error, /several Capella models/);
});

test('a .json is imported only when it lists API records', () => {
  const records = '[{"@id":"1f","@type":"PartDefinition","name":"Vehicle"}]';
  assert.equal(looksLikeApiRecords(records), true);
  assert.equal(looksLikeApiRecords('﻿  {"elements":[{"@type":"Package","@id":"x"}]}'), true);
  assert.equal(looksLikeApiRecords('{"name":"arclang","version":"4.5.0"}'), false);
  assert.equal(looksLikeApiRecords('not json'), false);
  assert.equal((resolveFile('/a/commit.json', [], records) as { format: string }).format, 'sysml');
  assert.match((resolveFile('/a/package.json', [], '{"name":"x"}') as { error: string }).error, /not a listing of SysML v2 API records/);
});

test('a folder offers its Capella models and its SysML v2 files as one model', () => {
  const capella = folderImports('/w/test7_0', [{ name: 'Model Test 7.0.capella', isDirectory: false }, { name: 'Model Test 7.0.aird', isDirectory: false }, { name: '.project', isDirectory: false }], false);
  assert.equal(capella.length, 1);
  assert.deepEqual([capella[0].format, capella[0].input, capella[0].base], ['capella', '/w/test7_0/Model Test 7.0.capella', 'Model Test 7.0']);
  const sysml = folderImports('/w/vehicle/', [{ name: 'parts', isDirectory: true }], true);
  assert.deepEqual(sysml.map((o) => [o.format, o.input, o.base]), [['sysml', '/w/vehicle/', 'vehicle']]);
  assert.equal(folderImports('/w/empty', [{ name: 'readme.md', isDirectory: false }], false).length, 0);
  assert.equal(folderImports('/w/both', [{ name: 'M.capella', isDirectory: false }], true).length, 2);
});

test('several SysML files chosen: their folder when they are all of it, else a copy', () => {
  const entries = [{ name: 'a.sysml', isDirectory: false }, { name: 'b.sysml', isDirectory: false }, { name: 'readme.md', isDirectory: false }];
  assert.deepEqual(sysmlSelection(['/m/a.sysml', '/m/b.sysml'], entries), { folder: '/m', copy: false });
  assert.deepEqual(sysmlSelection(['/m/a.sysml'], entries), { folder: '/m', copy: true });
  assert.deepEqual(sysmlSelection(['/m/a.sysml', '/n/c.sysml'], entries), { folder: '', copy: true });
  assert.deepEqual(sysmlSelection(['/m/a.sysml', '/m/b.sysml'], [...entries, { name: 'lib', isDirectory: true }]), { folder: '/m', copy: true });
  assert.match((sysmlSelection(['/m/a.sysml', '/m/r.reqif'], entries) as { error: string }).error, /r\.reqif/);
  assert.equal(selectionBase(['/m/vehicle/a.sysml', '/m/vehicle/b.sysml']), 'vehicle');
  assert.equal(selectionBase(['/m/a.sysml', '/n/b.sysml']), 'a');
});

test('the compiler output is read back into counts, left out, notes and the refusal', () => {
  const stdout = [
    'Importing from Capella: /x/M.capella...',
    '  56 allocation traces',
    '  1 roles (left out: ArcLang has no operational role)',
    '  3 requirement relations to elements not written (left out)',
    '  4 scenarios without a message',
    '  note: a diagram of a chain is listed, not drawn',
    '✓ Import successful',
    '  Input: /x/M.capella',
    '  Output: m.arc',
  ].join('\n');
  const outcome = parseImportOutput(stdout, '', 0);
  assert.equal(outcome.ok, true);
  assert.deepEqual(outcome.counts.map((c) => [c.count, c.leftOut]), [[56, false], [1, true], [3, true], [4, false]]);
  assert.deepEqual(outcome.notes, ['a diagram of a chain is listed, not drawn']);
  assert.deepEqual(countTotals(outcome.counts), { imported: 60, leftOut: 4 });
  const refused = parseImportOutput('Importing from SysML: v.sysml...\n', '  the refused text is in /x/v.rejected.arc\nError: Compilation error: the written model does not compile: Unknown reference "Foo"\n', 1);
  assert.equal(refused.ok, false);
  assert.equal(refused.rejected, '/x/v.rejected.arc');
  assert.match(refused.error ?? '', /^the written model does not compile/);
  assert.equal(parseImportOutput('', 'boom', 2).error, 'boom');
  assert.deepEqual(parseImportOutput('', 'warning: ReqIF carries requirements only', 0).warnings, ['ReqIF carries requirements only']);
});

test('an API server is https, or http on this machine, with no credentials in the address', () => {
  assert.deepEqual(checkApiBase('https://sysml.example.org/api/'), { base: 'https://sysml.example.org/api', local: false });
  assert.deepEqual(checkApiBase('http://localhost:5590/api/sysml-v2'), { base: 'http://localhost:5590/api/sysml-v2', local: true });
  assert.deepEqual(checkApiBase('http://127.0.0.1:9000'), { base: 'http://127.0.0.1:9000', local: true });
  assert.match((checkApiBase('http://sysml.example.org/api') as { error: string }).error, /https/);
  assert.match((checkApiBase('https://me:secret@sysml.example.org') as { error: string }).error, /no user or password/);
  assert.match((checkApiBase('ftp://x') as { error: string }).error, /https/);
  assert.match((checkApiBase('nope') as { error: string }).error, /not an address/);
  assert.match((checkApiBase('https://x/api?token=1') as { error: string }).error, /no query/);
});

test('pages follow the Link header, on the same server only', () => {
  const base = 'http://localhost:5590/api/sysml-v2/projects/p/commits/c/elements?page[size]=2';
  const link = '</api/sysml-v2/projects/p/commits/c/elements?page[after]=e2&page[size]=2>; rel="next", </api/sysml-v2/projects/p/commits/c/elements?page[before]=e1&page[size]=2>; rel="prev"';
  assert.equal(nextPage(link, base), 'http://localhost:5590/api/sysml-v2/projects/p/commits/c/elements?page[after]=e2&page[size]=2');
  assert.equal(nextPage('</x?page[before]=a>; rel="prev"', base), null);
  assert.equal(nextPage(null, base), null);
  assert.match((nextPage('<https://evil.example/steal>; rel="next"', base) as { error: string }).error, /another server/);
  assert.equal(withPageSize('http://h/projects', 100), 'http://h/projects?page[size]=100');
  assert.equal(withPageSize('http://h/x?a=1', 5), 'http://h/x?a=1&page[size]=5');
});

test('projects and commits are listed by name, newest first', () => {
  assert.deepEqual(recordLabel({ '@id': 'p1', name: 'AEB', description: 'brakes', created: '2026-10-20' }), { id: 'p1', label: 'AEB', detail: '2026-10-20 · brakes · p1' });
  assert.equal(recordLabel({ '@id': 'c1', name: null, description: 'Compiled snapshot' })?.label, 'Compiled snapshot');
  assert.equal(recordLabel({ name: 'no id' }), null);
  assert.deepEqual(newestFirst([{ '@id': 'a', created: '2026-01-01T00:00:00Z' }, { '@id': 'b', created: '2026-06-01T00:00:00Z' }]).map((c) => c['@id']), ['b', 'a']);
});

test('a .reqifz gives up its ReqIF document', () => {
  const text = new TextEncoder().encode('<REQ-IF/>');
  const archive = zip([{ name: 'readme.txt', data: new TextEncoder().encode('hi') }, { name: 'module.reqif', data: text }], (d) => new Uint8Array(zlib.deflateRawSync(d)));
  const entries = unzip(archive, (d) => new Uint8Array(zlib.inflateRawSync(d)));
  const entry = reqifEntryOf(entries);
  assert.equal((entry as { name: string }).name, 'module.reqif');
  assert.equal(new TextDecoder().decode((entry as { data: Uint8Array }).data), '<REQ-IF/>');
  assert.match((reqifEntryOf([{ name: 'a.txt' }]) as { error: string }).error, /no \.reqif document/);
  assert.match((reqifEntryOf([{ name: '../evil.reqif' }]) as { error: string }).error, /no \.reqif/);
});

test('model files dropped on the sheet are found in what the drag carries', () => {
  const list = 'file:///w/Model%20Test%207.0.capella\r\n# a comment\r\nfile:///w/notes.txt\r\n';
  assert.deepEqual(droppedModelFiles([list]), ['file:///w/Model%20Test%207.0.capella']);
  assert.deepEqual(droppedModelFiles(['["file:///w/v.sysml","file:///w/v.sysml"]']), ['file:///w/v.sysml']);
  assert.deepEqual(droppedModelFiles(['[{"resource":{"$mid":1,"scheme":"file","path":"/w/r.reqif"}}]']), ['file:///w/r.reqif']);
  assert.deepEqual(droppedModelFiles(['file:///w/m.arc#arclang-element=LC-1', 'https://x/m.sysml', '']), []);
});

test('requirements merged into a model: the declarations after the model block', () => {
  const text = '// Imported from ReqIF by ArcLang\nmodel Doors { note: "a { brace" }\n\nsystem_analysis "MODULE-1" {\n    req "R-1" { description: "x } y" }\n}\n';
  assert.equal(declarationsAfterModel(text), 'system_analysis "MODULE-1" {\n    req "R-1" { description: "x } y" }\n}');
  assert.equal(declarationsAfterModel('system_analysis "S" {}'), null);
  assert.equal(declarationsAfterModel('model M {\n'), null);
});
