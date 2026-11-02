import assert from 'node:assert/strict';
import { test } from 'node:test';
import { buildPdf, fitDrawing, jpegSize, layoutLegend, PAGE, pdfText, rgbOf, textWidth } from '../../src/pure/pdf';

// A 4 x 2 red JPEG.
const JPEG = new Uint8Array(Buffer.from('/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAACAAQDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDkKKKK8U/TD//Z', 'base64'));

const text = (pdf: Uint8Array) => Buffer.from(pdf).toString('latin1');

test('the size of a JPEG is read from its frame header', () => {
  assert.deepEqual(jpegSize(JPEG), { width: 4, height: 2, components: 3 });
  assert.equal(jpegSize(new Uint8Array([0x89, 0x50, 0x4e, 0x47])), null);
});

test('one page per diagram, with its title, its drawing, its legend and the page number', () => {
  const legend = [
    { label: 'Logical component', fill: '#dbe8fb', stroke: '#2f5ea8' },
    { label: 'Component exchange', stroke: 'rgb(30, 60, 140)', line: true },
    { label: 'Physical exchange', stroke: '#2f5ea8', line: true, dash: true },
  ];
  const pdf = buildPdf({ title: 'Cooling — hall', pages: [{ title: 'Logical — Cooling plant', jpeg: JPEG, legend }, { title: 'Physical', jpeg: JPEG, legend: [] }] });
  const s = text(pdf);
  assert.ok(s.startsWith('%PDF-1.4'));
  assert.match(s, /\/Type \/Pages \/Count 2 /);
  assert.equal((s.match(/\/Type \/Page /g) ?? []).length, 2);
  assert.equal((s.match(/\/Filter \/DCTDecode/g) ?? []).length, 2);
  assert.ok(s.includes(pdfText('Logical — Cooling plant')), 'the title of the first page');
  assert.ok(s.includes(pdfText('Component exchange')), 'the legend');
  assert.ok(s.includes('[3 2] 0 d'), 'a dashed line kind');
  assert.ok(s.includes(pdfText('1 / 2')) && s.includes(pdfText('2 / 2')), 'the page numbers');
  assert.ok(s.includes(`/Title ${pdfText('Cooling — hall')}`), 'the document title');
  // The cross-reference table points at every object, and startxref at the table.
  const xref = Number(/startxref\n(\d+)\n%%EOF\n$/.exec(s)?.[1]);
  assert.ok(s.slice(xref).startsWith('xref\n'));
  const entries = [...s.slice(xref).matchAll(/^(\d{10}) 00000 n $/gm)].map((m) => Number(m[1]));
  entries.forEach((offset, i) => assert.ok(s.slice(offset).startsWith(`${i + 1} 0 obj\n`), `object ${i + 1}`));
  // Each content stream's /Length is its real length.
  for (const m of s.matchAll(/<< \/Length (\d+) >>\nstream\n/g)) {
    const start = (m.index ?? 0) + m[0].length;
    assert.equal(s.slice(start + Number(m[1]), start + Number(m[1]) + 10), '\nendstream');
  }
});

test('a page whose drawing is not a JPEG is refused, with its title', () => {
  assert.throws(() => buildPdf({ title: 'M', pages: [{ title: 'Logical', jpeg: new Uint8Array([1, 2, 3]), legend: [] }] }), /page 1 \(Logical\): the drawing is not a JPEG/);
});

test('text is WinAnsi in hex, colors are read from CSS', () => {
  assert.equal(pdfText('Aé—'), '<41e997>');
  assert.equal(pdfText('日'), '<3f>');
  assert.deepEqual(rgbOf('#fff'), [1, 1, 1]);
  assert.deepEqual(rgbOf('#000000'), [0, 0, 0]);
  assert.deepEqual(rgbOf('rgba(255, 0, 0, 0.5)'), [1, 0, 0]);
  assert.equal(rgbOf('none'), null);
  assert.deepEqual(rgbOf('oklch(50% 0.1 250)'), [0.5, 0.5, 0.5]);
  assert.ok(Math.abs(textWidth('Hello', 10) - 22.78) < 0.01);
});

test('the legend wraps to the page and the drawing keeps its proportions above it', () => {
  const items = Array.from({ length: 30 }, (_, i) => ({ label: `Kind number ${i}`, stroke: '#000' }));
  const laid = layoutLegend(items, PAGE.width - 72);
  assert.ok(laid.lines > 1);
  assert.ok(laid.placed.every((p) => p.x + 12 <= PAGE.width - 72));
  const box = fitDrawing(1600, 900, laid.lines);
  assert.ok(Math.abs(box.w / box.h - 16 / 9) < 0.01);
  assert.ok(box.x >= 36 && box.x + box.w <= PAGE.width - 36 + 0.01);
  assert.equal(layoutLegend([], 800).lines, 0);
});
