import assert from 'node:assert/strict';
import { test } from 'node:test';
import * as node from 'node:zlib';
import { deflateRawSync, gunzipSync, gzipSync, inflateRawSync } from '../../src/web/shims/zlib';

test('the browser build\'s zlib deflates and inflates as node:zlib does', () => {
  const text = 'ArcLang '.repeat(5000);
  const packed = deflateRawSync(text);
  assert.ok(packed.length < 1000);
  assert.equal(node.inflateRawSync(packed).toString(), text, 'node reads what the browser wrote');
  assert.equal(inflateRawSync(node.deflateRawSync(Buffer.from(text))).toString(), text, 'the browser reads what node wrote');
  assert.equal(gunzipSync(gzipSync(text)).toString(), text);
  assert.throws(() => inflateRawSync(packed, { maxOutputLength: 100 }), /larger than 100 bytes/);
});
