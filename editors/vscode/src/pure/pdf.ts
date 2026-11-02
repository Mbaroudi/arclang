// A PDF of the diagrams: one 16:9 page per diagram, its title on top, the
// drawing fitted below it, the legend of what it shows at the bottom, the
// model's name and the page number in the footer. Written by hand (no
// dependency): the drawing is a JPEG the PDF embeds as it is, the text is
// in the base fonts every reader has (Helvetica), so the file is small and
// opens anywhere.

/** One entry of a diagram's legend: a box kind (fill and stroke) or a line kind. */
export interface LegendItem {
  label: string;
  stroke: string;
  fill?: string;
  /** A line kind: drawn as a stroke, dashed or not. */
  line?: boolean;
  dash?: boolean;
}

export interface PdfPage {
  title: string;
  /** The drawing, a baseline or progressive JPEG. */
  jpeg: Uint8Array;
  legend: LegendItem[];
}

export interface PdfDocument {
  /** The model's name: the document title and the footer. */
  title: string;
  pages: PdfPage[];
}

/** A 16:9 page, in points. */
export const PAGE = { width: 960, height: 540 };
const MARGIN = 36;
const TITLE_SIZE = 20;
const LEGEND_SIZE = 9;
const FOOTER_SIZE = 8;
const SWATCH = 12;

/**
 * Widths of the Helvetica glyphs 32 to 126, in thousandths of the font
 * size (the Adobe metrics): what a legend line needs to wrap.
 */
const HELVETICA = [278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584];

/** The width of `text` in Helvetica at `size` points. */
export function textWidth(text: string, size: number): number {
  let units = 0;
  for (const ch of text) {
    const code = ch.codePointAt(0) ?? 63;
    units += code >= 32 && code <= 126 ? HELVETICA[code - 32] : 556;
  }
  return (units * size) / 1000;
}

/** The characters WinAnsi places above 127 that are not Latin-1. */
const WIN_ANSI: Record<string, number> = { '€': 0x80, '‚': 0x82, '„': 0x84, '…': 0x85, '‘': 0x91, '’': 0x92, '“': 0x93, '”': 0x94, '•': 0x95, '–': 0x96, '—': 0x97, '™': 0x99 };

/** `text` as a PDF hex string in WinAnsi: what a base font can show, `?` for the rest. */
export function pdfText(text: string): string {
  let hex = '';
  for (const ch of text) {
    const code = ch.codePointAt(0) ?? 63;
    const byte = WIN_ANSI[ch] ?? (code >= 32 && code <= 126 ? code : code >= 160 && code <= 255 ? code : 63);
    hex += byte.toString(16).padStart(2, '0');
  }
  return `<${hex}>`;
}

/** A CSS color as PDF RGB components (0 to 1): #rgb, #rrggbb, rgb(), rgba(); gray when unreadable. */
export function rgbOf(color: string | undefined): [number, number, number] | null {
  if (!color || color === 'none' || color === 'transparent') return null;
  const c = color.trim().toLowerCase();
  let m = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/.exec(c);
  if (m) return [m[1], m[2], m[3]].map((h) => parseInt(h + h, 16) / 255) as [number, number, number];
  m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})(?:[0-9a-f]{2})?$/.exec(c);
  if (m) return [m[1], m[2], m[3]].map((h) => parseInt(h, 16) / 255) as [number, number, number];
  m = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/.exec(c);
  if (m) return [m[1], m[2], m[3]].map((v) => Math.min(255, Number(v)) / 255) as [number, number, number];
  return [0.5, 0.5, 0.5];
}

/** The size of a JPEG, read from its frame header; null when it is not one. */
export function jpegSize(data: Uint8Array): { width: number; height: number; components: number } | null {
  if (data[0] !== 0xff || data[1] !== 0xd8) return null;
  let i = 2;
  while (i + 9 < data.length) {
    if (data[i] !== 0xff) return null;
    const marker = data[i + 1];
    const length = (data[i + 2] << 8) | data[i + 3];
    // SOF0 to SOF15, but not DHT (C4), JPG (C8) or DAC (CC).
    if (marker >= 0xc0 && marker <= 0xcf && marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc) {
      return { height: (data[i + 5] << 8) | data[i + 6], width: (data[i + 7] << 8) | data[i + 8], components: data[i + 9] };
    }
    i += 2 + length;
  }
  return null;
}

/** The legend laid out in lines under the drawing: [x, y] of each item, from the bottom up. */
export function layoutLegend(items: LegendItem[], width: number): { placed: { item: LegendItem; x: number; line: number }[]; lines: number } {
  const placed: { item: LegendItem; x: number; line: number }[] = [];
  let x = 0;
  let line = 0;
  for (const item of items) {
    const w = SWATCH + 6 + textWidth(item.label, LEGEND_SIZE) + 18;
    if (x > 0 && x + w > width) {
      line += 1;
      x = 0;
    }
    placed.push({ item, x, line });
    x += w;
  }
  return { placed, lines: items.length ? line + 1 : 0 };
}

/** Where the drawing goes: as large as fits between the title and the legend, centered. */
export function fitDrawing(width: number, height: number, legendLines: number): { x: number; y: number; w: number; h: number } {
  const top = PAGE.height - MARGIN - TITLE_SIZE - 14;
  const bottom = MARGIN + FOOTER_SIZE + 10 + (legendLines ? legendLines * (LEGEND_SIZE + 9) + 8 : 0);
  const boxW = PAGE.width - 2 * MARGIN;
  const boxH = Math.max(40, top - bottom);
  const scale = Math.min(boxW / width, boxH / height);
  const w = width * scale;
  const h = height * scale;
  return { x: MARGIN + (boxW - w) / 2, y: bottom + (boxH - h) / 2, w, h };
}

const f = (n: number) => (Math.round(n * 100) / 100).toString();

function pageContent(page: PdfPage, index: number, total: number, doc: PdfDocument, size: { width: number; height: number }): string {
  const ops: string[] = [];
  // Title.
  ops.push(`BT /F2 ${TITLE_SIZE} Tf 0.13 0.14 0.17 rg ${MARGIN} ${f(PAGE.height - MARGIN - TITLE_SIZE)} Td ${pdfText(page.title)} Tj ET`);
  ops.push(`0.8 0.8 0.8 RG 0.75 w ${MARGIN} ${f(PAGE.height - MARGIN - TITLE_SIZE - 8)} m ${PAGE.width - MARGIN} ${f(PAGE.height - MARGIN - TITLE_SIZE - 8)} l S`);
  // Legend.
  const legend = layoutLegend(page.legend, PAGE.width - 2 * MARGIN);
  const legendTop = MARGIN + FOOTER_SIZE + 10 + legend.lines * (LEGEND_SIZE + 9);
  for (const { item, x, line } of legend.placed) {
    const left = MARGIN + x;
    const base = legendTop - (line + 1) * (LEGEND_SIZE + 9) + 4;
    const stroke = rgbOf(item.stroke) ?? [0.4, 0.4, 0.4];
    if (item.line) {
      ops.push(`q ${stroke.map(f).join(' ')} RG 1.6 w ${item.dash ? '[3 2] 0 d' : '[] 0 d'} ${f(left)} ${f(base + 3)} m ${f(left + SWATCH)} ${f(base + 3)} l S Q`);
    } else {
      const fill = rgbOf(item.fill);
      ops.push(`q ${(fill ?? [1, 1, 1]).map(f).join(' ')} rg ${stroke.map(f).join(' ')} RG 0.8 w ${f(left)} ${f(base - 2)} ${SWATCH} ${SWATCH - 3} re B Q`);
    }
    ops.push(`BT /F1 ${LEGEND_SIZE} Tf 0.25 0.27 0.3 rg ${f(left + SWATCH + 6)} ${f(base)} Td ${pdfText(item.label)} Tj ET`);
  }
  // Drawing.
  const box = fitDrawing(size.width, size.height, legend.lines);
  ops.push(`q ${f(box.w)} 0 0 ${f(box.h)} ${f(box.x)} ${f(box.y)} cm /Im1 Do Q`);
  // Footer.
  const pageNumber = `${index + 1} / ${total}`;
  ops.push(`BT /F1 ${FOOTER_SIZE} Tf 0.5 0.5 0.5 rg ${MARGIN} ${MARGIN - 12} Td ${pdfText(doc.title)} Tj ET`);
  ops.push(`BT /F1 ${FOOTER_SIZE} Tf 0.5 0.5 0.5 rg ${f(PAGE.width - MARGIN - textWidth(pageNumber, FOOTER_SIZE))} ${MARGIN - 12} Td ${pdfText(pageNumber)} Tj ET`);
  return ops.join('\n');
}

/** The PDF file of `doc`. Throws when a page's drawing is not a JPEG. */
export function buildPdf(doc: PdfDocument): Uint8Array {
  const encoder = new TextEncoder();
  const chunks: Uint8Array[] = [];
  let length = 0;
  const offsets: number[] = [];
  const write = (part: string | Uint8Array) => {
    const bytes = typeof part === 'string' ? encoder.encode(part) : part;
    chunks.push(bytes);
    length += bytes.length;
  };
  const object = (id: number, body: string | (() => void)) => {
    offsets[id] = length;
    write(`${id} 0 obj\n`);
    if (typeof body === 'string') write(body);
    else body();
    write('\nendobj\n');
  };

  // Ids: 1 catalog, 2 pages, 3 Helvetica, 4 Helvetica-Bold, 5 info, then 3 per page (page, content, image).
  const pageIds = doc.pages.map((_, i) => 6 + 3 * i);
  write('%PDF-1.4\n%\u00e2\u00e3\u00cf\u00d3\n'); // a comment of bytes above 127: the file is binary
  object(1, '<< /Type /Catalog /Pages 2 0 R >>');
  object(2, `<< /Type /Pages /Count ${doc.pages.length} /Kids [${pageIds.map((id) => `${id} 0 R`).join(' ')}] >>`);
  object(3, '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>');
  object(4, '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>');
  object(5, `<< /Title ${pdfText(doc.title)} /Producer (ArcLang) /Creator (ArcLang) >>`);
  doc.pages.forEach((page, i) => {
    const size = jpegSize(page.jpeg);
    if (!size) throw new Error(`page ${i + 1} (${page.title}): the drawing is not a JPEG`);
    const [pageId, contentId, imageId] = [pageIds[i], pageIds[i] + 1, pageIds[i] + 2];
    const content = pageContent(page, i, doc.pages.length, doc, size);
    object(pageId, `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${PAGE.width} ${PAGE.height}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << /Im1 ${imageId} 0 R >> >> /Contents ${contentId} 0 R >>`);
    object(contentId, `<< /Length ${encoder.encode(content).length} >>\nstream\n${content}\nendstream`);
    object(imageId, () => {
      const space = size.components === 1 ? '/DeviceGray' : size.components === 4 ? '/DeviceCMYK' : '/DeviceRGB';
      write(`<< /Type /XObject /Subtype /Image /Width ${size.width} /Height ${size.height} /ColorSpace ${space} /BitsPerComponent 8 /Filter /DCTDecode /Length ${page.jpeg.length} >>\nstream\n`);
      write(page.jpeg);
      write('\nendstream');
    });
  });
  const objects = 5 + 3 * doc.pages.length;
  const xref = length;
  write(`xref\n0 ${objects + 1}\n0000000000 65535 f \n`);
  for (let id = 1; id <= objects; id++) write(`${String(offsets[id]).padStart(10, '0')} 00000 n \n`);
  write(`trailer\n<< /Size ${objects + 1} /Root 1 0 R /Info 5 0 R >>\nstartxref\n${xref}\n%%EOF\n`);

  const out = new Uint8Array(length);
  let at = 0;
  for (const chunk of chunks) {
    out.set(chunk, at);
    at += chunk.length;
  }
  return out;
}
