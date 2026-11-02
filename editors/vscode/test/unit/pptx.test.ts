import assert from 'node:assert/strict';
import { test } from 'node:test';
import { deflateRawSync, deflateSync, inflateRawSync } from 'node:zlib';
import { buildPptx, fitPicture, pngSize, xmlText } from '../../src/pure/pptx';
import { crc32, unzip, zip } from '../../src/pure/zip';

/** A valid PNG of `w` x `h` pixels, white (built with zlib, as a browser canvas would give one). */
function png(w: number, h: number): Uint8Array {
  const chunk = (type: string, data: Buffer) => {
    const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
    const length = Buffer.alloc(4);
    length.writeUInt32BE(data.length);
    const crc = Buffer.alloc(4);
    crc.writeUInt32BE(crc32(body));
    return Buffer.concat([length, body, crc]);
  };
  const header = Buffer.alloc(13);
  header.writeUInt32BE(w, 0);
  header.writeUInt32BE(h, 4);
  header[8] = 8; // bit depth
  header[9] = 2; // RGB
  const rows = Buffer.alloc((w * 3 + 1) * h, 0xff);
  for (let y = 0; y < h; y += 1) rows[y * (w * 3 + 1)] = 0;
  return new Uint8Array(Buffer.concat([Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), chunk('IHDR', header), chunk('IDAT', deflateSync(rows)), chunk('IEND', Buffer.alloc(0))]));
}

const text = (entries: { name: string; data: Uint8Array }[], name: string) => new TextDecoder().decode(entries.find((e) => e.name === name)!.data);

test('CRC-32 is the one of ISO 3309', () => {
  assert.equal(crc32(new TextEncoder().encode('123456789')), 0xcbf43926);
  assert.equal(crc32(new Uint8Array()), 0);
});

test('a ZIP reads back entry by entry, stored or deflated, and refuses unsafe names', () => {
  const entries = [{ name: 'a.txt', data: new TextEncoder().encode('hello '.repeat(200)) }, { name: 'dir/b.bin', data: new Uint8Array([1, 2, 3]) }];
  for (const deflate of [undefined, (d: Uint8Array) => new Uint8Array(deflateRawSync(d))]) {
    const back = unzip(zip(entries, deflate), (d) => new Uint8Array(inflateRawSync(d)));
    assert.deepEqual(back.map((e) => e.name), ['a.txt', 'dir/b.bin']);
    assert.deepEqual(back[0].data, entries[0].data);
    assert.deepEqual(back[1].data, entries[1].data);
  }
  assert.ok(zip(entries, (d) => new Uint8Array(deflateRawSync(d))).length < zip(entries).length, 'text is deflated');
  assert.throws(() => zip([{ name: '../evil', data: new Uint8Array() }]), /bad or repeated/);
  assert.throws(() => zip([{ name: 'a', data: new Uint8Array() }, { name: 'a', data: new Uint8Array() }]), /bad or repeated/);
});

test('a picture is as large as fits under the title, centered, its proportions kept', () => {
  const wide = fitPicture(4000, 1000);
  assert.ok(wide.x >= 457200 && wide.x + wide.cx <= 12192000 - 457200 + 1);
  assert.ok(Math.abs(wide.cx / wide.cy - 4) < 0.01);
  const tall = fitPicture(800, 2400);
  assert.ok(tall.y >= 900000 && tall.y + tall.cy <= 6858000);
  assert.ok(Math.abs(tall.cx / tall.cy - 1 / 3) < 0.01);
  assert.ok(tall.x > wide.x, 'a tall picture is centered across');
});

test('a deck has one slide, one picture and one notes page per diagram, every part declared', () => {
  const deck = buildPptx({
    title: 'Lane keeping <assist>',
    slides: [
      { title: 'Logical — LKA', png: png(40, 20), width: 40, height: 20, notes: ['Logical component: Camera', 'Component exchange: LaneFlow & more'] },
      { title: 'Physical', png: png(30, 30), width: 30, height: 30, notes: [] },
    ],
  }, (d) => new Uint8Array(deflateRawSync(d)));
  const entries = unzip(deck, (d) => new Uint8Array(inflateRawSync(d)));
  const names = entries.map((e) => e.name);
  for (const part of ['[Content_Types].xml', '_rels/.rels', 'ppt/presentation.xml', 'ppt/slides/slide1.xml', 'ppt/slides/slide2.xml', 'ppt/notesSlides/notesSlide2.xml', 'ppt/media/image1.png', 'ppt/theme/theme1.xml', 'ppt/notesMasters/notesMaster1.xml']) {
    assert.ok(names.includes(part), part);
  }
  const types = text(entries, '[Content_Types].xml');
  for (const name of names.filter((n) => n.endsWith('.xml') && n !== '[Content_Types].xml')) assert.ok(types.includes(`PartName="/${name}"`), `${name} declared`);
  assert.match(text(entries, 'ppt/presentation.xml'), /<p:sldSz cx="12192000" cy="6858000"\/>/);
  assert.equal((text(entries, 'ppt/presentation.xml').match(/<p:sldId /g) ?? []).length, 2);
  assert.match(text(entries, 'ppt/slides/slide1.xml'), /<a:t>Logical — LKA<\/a:t>/);
  assert.match(text(entries, 'ppt/notesSlides/notesSlide1.xml'), /Component exchange: LaneFlow &amp; more/);
  assert.match(text(entries, 'docProps/core.xml'), /Lane keeping &lt;assist&gt;/);
  assert.deepEqual(entries.find((e) => e.name === 'ppt/media/image2.png')!.data, png(30, 30));
});

test('a picture that is not a PNG, or no slide at all, is refused', () => {
  assert.throws(() => buildPptx({ title: 't', slides: [] }), /at least one slide/);
  assert.throws(() => buildPptx({ title: 't', slides: [{ title: 'x', png: new Uint8Array([1, 2, 3]), width: 1, height: 1, notes: [] }] }), /not a PNG/);
});

test('XML text drops forbidden characters and escapes markup', () => {
  assert.equal(xmlText('a<b>&"c"\u0001'), 'a&lt;b&gt;&amp;&quot;c&quot;');
  assert.deepEqual(pngSize(png(12, 7)), { width: 12, height: 7 });
  assert.equal(pngSize(new Uint8Array(4)), null);
});

test('each slide carries the legend of its diagram, and the picture leaves it room', () => {
  const legend = [
    { label: 'Logical component', fill: '#dbe8fb', stroke: '#2f5ea8' },
    { label: 'Component exchange', stroke: '#1e3c8c', line: true },
    { label: 'Physical exchange', stroke: '#2f5ea8', line: true, dash: true },
  ];
  const deck = buildPptx({ title: 'Cooling', slides: [{ title: 'Logical', png: png(16, 9), width: 16, height: 9, notes: [], legend }, { title: 'Bare', png: png(16, 9), width: 16, height: 9, notes: [] }] });
  const entries = unzip(deck, (d) => new Uint8Array(inflateRawSync(d)));
  const first = text(entries, 'ppt/slides/slide1.xml');
  for (const label of ['Logical component', 'Component exchange', 'Physical exchange']) assert.ok(first.includes(`<a:t>${label}</a:t>`), label);
  assert.ok(first.includes('<a:srgbClr val="DBE8FB"/>'), 'the box swatch takes the kind\'s fill');
  assert.ok(first.includes('<a:prstDash val="dash"/>'), 'a dashed line kind');
  assert.ok(!text(entries, 'ppt/slides/slide2.xml').includes('Legend'), 'no legend when the diagram has none');
  // The picture is smaller with a legend under it, never over it.
  assert.ok(fitPicture(1600, 900, 1).cy < fitPicture(1600, 900, 0).cy);
  const ids = [...first.matchAll(/<p:cNvPr id="(\d+)"/g)].map((m) => m[1]);
  assert.equal(new Set(ids).size, ids.length, 'every shape has its own id');
});
