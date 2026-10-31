import assert from 'node:assert/strict';
import { test } from 'node:test';
import { DOCUMENT_FORMATS, documentArgs } from '../../src/pure/document';

test('a document is asked of the compiler with its template, if any', () => {
  assert.deepEqual(documentArgs('/m/ev.arc', '/m/ev-review.docx', null), ['document', '/m/ev.arc', '-o', '/m/ev-review.docx']);
  assert.deepEqual(documentArgs('/m/ev.arc', '/m/r.html', '/m/review.md'), ['document', '/m/ev.arc', '-o', '/m/r.html', '--template', '/m/review.md']);
  assert.deepEqual(DOCUMENT_FORMATS.map((f) => f.id), ['docx', 'html', 'md']);
});
