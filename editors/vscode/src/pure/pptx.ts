// A PowerPoint file (.pptx, Office Open XML, ECMA-376) of diagrams: one 16:9
// slide per diagram, its title on top, the drawing as a picture fitted
// below it, and speaker notes listing what it shows. Written by hand on the
// ZIP writer: no Office library. The parts are the minimum PowerPoint,
// Keynote, LibreOffice and python-pptx open: a master, one "Title Only"
// layout, a theme, a notes master, and per slide its picture and notes.
import { Deflate, zip, ZipEntry } from './zip';

export interface Slide {
  title: string;
  /** The drawing, PNG bytes, and its size in pixels. */
  png: Uint8Array;
  width: number;
  height: number;
  /** One line per element shown, in the speaker notes. */
  notes: string[];
}

export interface Deck {
  title: string;
  author?: string;
  slides: Slide[];
}

// Sizes in EMU (914 400 per inch). A 16:9 slide is 13.333 x 7.5 in.
const SLIDE = { cx: 12192000, cy: 6858000 };
const NOTES = { cx: 6858000, cy: 9144000 };
const MARGIN = 457200; // 0.5 in
const TITLE = { y: 228600, cy: 685800 };
const PICTURE_TOP = TITLE.y + TITLE.cy + 114300;

const NS_A = 'http://schemas.openxmlformats.org/drawingml/2006/main';
const NS_R = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';
const NS_P = 'http://schemas.openxmlformats.org/presentationml/2006/main';
const REL = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';
const PKG_REL = 'http://schemas.openxmlformats.org/package/2006/relationships';
const CT = 'application/vnd.openxmlformats-officedocument.presentationml';
const XML = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n';

/** Text made safe in XML content and attributes; characters XML 1.0 forbids are dropped. */
export function xmlText(text: string): string {
  return String(text)
    // eslint-disable-next-line no-control-regex
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F￾￿]/g, '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/** Where a picture of `width` x `height` goes: as large as fits under the title, centered. */
export function fitPicture(width: number, height: number): { x: number; y: number; cx: number; cy: number } {
  const boxW = SLIDE.cx - MARGIN * 2;
  const boxH = SLIDE.cy - PICTURE_TOP - MARGIN / 2;
  const scale = Math.min(boxW / Math.max(1, width), boxH / Math.max(1, height));
  const cx = Math.round(width * scale);
  const cy = Math.round(height * scale);
  return { x: Math.round(MARGIN + (boxW - cx) / 2), y: Math.round(PICTURE_TOP + (boxH - cy) / 2), cx, cy };
}

const rels = (items: [string, string, string][]) =>
  `${XML}<Relationships xmlns="${PKG_REL}">${items.map(([id, type, target]) => `<Relationship Id="${id}" Type="${REL}/${type}" Target="${target}"/>`).join('')}</Relationships>`;

const group = '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>';
const titleBox = (id: number, body: string) =>
  `<p:sp><p:nvSpPr><p:cNvPr id="${id}" name="Title ${id}"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>` +
  `<p:spPr><a:xfrm><a:off x="${MARGIN}" y="${TITLE.y}"/><a:ext cx="${SLIDE.cx - MARGIN * 2}" cy="${TITLE.cy}"/></a:xfrm></p:spPr>${body}</p:sp>`;
const paragraph = (text: string, size?: number) =>
  `<a:p><a:r><a:rPr lang="en-GB"${size ? ` sz="${size}"` : ''} dirty="0"/><a:t>${xmlText(text)}</a:t></a:r></a:p>`;

function presentation(count: number): string {
  const slides = Array.from({ length: count }, (_, i) => `<p:sldId id="${256 + i}" r:id="rId${10 + i}"/>`).join('');
  return `${XML}<p:presentation xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}" saveSubsetFonts="1">` +
    '<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>' +
    '<p:notesMasterIdLst><p:notesMasterId r:id="rId2"/></p:notesMasterIdLst>' +
    `<p:sldIdLst>${slides}</p:sldIdLst>` +
    `<p:sldSz cx="${SLIDE.cx}" cy="${SLIDE.cy}"/><p:notesSz cx="${NOTES.cx}" cy="${NOTES.cy}"/>` +
    '<p:defaultTextStyle><a:lvl1pPr><a:defRPr sz="1800"><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:defaultTextStyle>' +
    '</p:presentation>';
}

function master(): string {
  return `${XML}<p:sldMaster xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}">` +
    `<p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree>${group}${titleBox(2, '<p:txBody><a:bodyPr anchor="b"/><a:lstStyle/><a:p><a:endParaRPr lang="en-GB"/></a:p></p:txBody>')}</p:spTree></p:cSld>` +
    '<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>' +
    '<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>' +
    '<p:txStyles>' +
    '<p:titleStyle><a:lvl1pPr algn="l"><a:defRPr sz="2800" b="1"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mj-lt"/></a:defRPr></a:lvl1pPr></p:titleStyle>' +
    '<p:bodyStyle><a:lvl1pPr><a:defRPr sz="1800"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:bodyStyle>' +
    '<p:otherStyle><a:lvl1pPr><a:defRPr sz="1800"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill></a:defRPr></a:lvl1pPr></p:otherStyle>' +
    '</p:txStyles></p:sldMaster>';
}

function layout(): string {
  return `${XML}<p:sldLayout xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}" type="titleOnly" preserve="1">` +
    `<p:cSld name="Title Only"><p:spTree>${group}${titleBox(2, '<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-GB"/></a:p></p:txBody>')}</p:spTree></p:cSld>` +
    '<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>';
}

function theme(name: string): string {
  const color = (tag: string, value: string) => `<a:${tag}><a:srgbClr val="${value}"/></a:${tag}>`;
  const solid = '<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>';
  const line = (w: number) => `<a:ln w="${w}" cap="flat" cmpd="sng" algn="ctr">${solid}<a:prstDash val="solid"/></a:ln>`;
  return `${XML}<a:theme xmlns:a="${NS_A}" name="${xmlText(name)}"><a:themeElements>` +
    `<a:clrScheme name="ArcLang">${color('dk1', '000000')}${color('lt1', 'FFFFFF')}${color('dk2', '18222D')}${color('lt2', 'E9EDF1')}` +
    `${color('accent1', '2A62A8')}${color('accent2', '3C8A3F')}${color('accent3', 'B0730C')}${color('accent4', '9C8500')}${color('accent5', 'C92A2A')}${color('accent6', '55616D')}` +
    `${color('hlink', '0B5CAD')}${color('folHlink', '6F42C1')}</a:clrScheme>` +
    '<a:fontScheme name="ArcLang"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>' +
    '<a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>' +
    `<a:fmtScheme name="ArcLang"><a:fillStyleLst>${solid}${solid}${solid}</a:fillStyleLst>` +
    `<a:lnStyleLst>${line(6350)}${line(12700)}${line(19050)}</a:lnStyleLst>` +
    '<a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst>' +
    `<a:bgFillStyleLst>${solid}${solid}${solid}</a:bgFillStyleLst></a:fmtScheme>` +
    '</a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>';
}

function notesMaster(): string {
  return `${XML}<p:notesMaster xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}"><p:cSld><p:spTree>${group}` +
    '<p:sp><p:nvSpPr><p:cNvPr id="2" name="Slide Image"/><p:cNvSpPr><a:spLocks noGrp="1" noRot="1" noChangeAspect="1"/></p:cNvSpPr><p:nvPr><p:ph type="sldImg" idx="2"/></p:nvPr></p:nvSpPr>' +
    '<p:spPr><a:xfrm><a:off x="381000" y="685800"/><a:ext cx="6096000" cy="3429000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:sp>' +
    '<p:sp><p:nvSpPr><p:cNvPr id="3" name="Notes"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="body" sz="quarter" idx="3"/></p:nvPr></p:nvSpPr>' +
    '<p:spPr><a:xfrm><a:off x="685800" y="4343400"/><a:ext cx="5486400" cy="4114800"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>' +
    '<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-GB"/></a:p></p:txBody></p:sp>' +
    '</p:spTree></p:cSld>' +
    '<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>' +
    '<p:notesStyle><a:lvl1pPr><a:defRPr sz="1200"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:notesStyle></p:notesMaster>';
}

function slide(item: Slide, index: number): string {
  const at = fitPicture(item.width, item.height);
  const body = `<p:txBody><a:bodyPr anchor="b"><a:normAutofit/></a:bodyPr><a:lstStyle/>${paragraph(item.title)}</p:txBody>`;
  return `${XML}<p:sld xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}"><p:cSld><p:spTree>${group}${titleBox(2, body)}` +
    `<p:pic><p:nvPicPr><p:cNvPr id="3" name="Diagram ${index + 1}" descr="${xmlText(item.title)}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr>` +
    `<p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>` +
    `<p:spPr><a:xfrm><a:off x="${at.x}" y="${at.y}"/><a:ext cx="${at.cx}" cy="${at.cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>` +
    '</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>';
}

function notesSlide(item: Slide): string {
  const lines = item.notes.length ? item.notes : ['(nothing drawn)'];
  return `${XML}<p:notes xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}"><p:cSld><p:spTree>${group}` +
    '<p:sp><p:nvSpPr><p:cNvPr id="2" name="Slide Image"/><p:cNvSpPr><a:spLocks noGrp="1" noRot="1" noChangeAspect="1"/></p:cNvSpPr><p:nvPr><p:ph type="sldImg"/></p:nvPr></p:nvSpPr><p:spPr/></p:sp>' +
    '<p:sp><p:nvSpPr><p:cNvPr id="3" name="Notes"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/>' +
    `<p:txBody><a:bodyPr/><a:lstStyle/>${paragraph(`Shown on ${item.title}:`)}${lines.map((line) => paragraph(line)).join('')}</p:txBody></p:sp>` +
    '</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:notes>';
}

function contentTypes(count: number): string {
  const over = (part: string, type: string) => `<Override PartName="${part}" ContentType="${type}"/>`;
  const slides = Array.from({ length: count }, (_, i) =>
    over(`/ppt/slides/slide${i + 1}.xml`, `${CT}.slide+xml`) + over(`/ppt/notesSlides/notesSlide${i + 1}.xml`, `${CT}.notesSlide+xml`)).join('');
  return `${XML}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">` +
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
    '<Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/>' +
    over('/ppt/presentation.xml', `${CT}.presentation.main+xml`) +
    over('/ppt/slideMasters/slideMaster1.xml', `${CT}.slideMaster+xml`) +
    over('/ppt/slideLayouts/slideLayout1.xml', `${CT}.slideLayout+xml`) +
    over('/ppt/notesMasters/notesMaster1.xml', `${CT}.notesMaster+xml`) +
    over('/ppt/theme/theme1.xml', 'application/vnd.openxmlformats-officedocument.theme+xml') +
    over('/ppt/theme/theme2.xml', 'application/vnd.openxmlformats-officedocument.theme+xml') +
    over('/ppt/presProps.xml', `${CT}.presProps+xml`) +
    over('/ppt/viewProps.xml', `${CT}.viewProps+xml`) +
    over('/ppt/tableStyles.xml', `${CT}.tableStyles+xml`) +
    over('/docProps/core.xml', 'application/vnd.openxmlformats-package.core-properties+xml') +
    over('/docProps/app.xml', 'application/vnd.openxmlformats-officedocument.extended-properties+xml') +
    slides + '</Types>';
}

function isPng(data: Uint8Array): boolean {
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  return data.length > 8 && signature.every((byte, i) => data[i] === byte);
}

/** The deck as .pptx bytes. Every slide's picture must be a PNG. */
export function buildPptx(deck: Deck, deflate?: Deflate): Uint8Array {
  if (deck.slides.length === 0) throw new Error('a presentation needs at least one slide');
  const text = (value: string) => new TextEncoder().encode(value);
  const entries: ZipEntry[] = [];
  const add = (name: string, value: string | Uint8Array) => entries.push({ name, data: typeof value === 'string' ? text(value) : value });
  const count = deck.slides.length;
  add('[Content_Types].xml', contentTypes(count));
  add('_rels/.rels', rels([
    ['rId1', 'officeDocument', 'ppt/presentation.xml'],
    ['rId2', 'metadata/core-properties', 'docProps/core.xml'],
    ['rId3', 'extended-properties', 'docProps/app.xml'],
  ]).replace(`${REL}/metadata/core-properties`, 'http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties'));
  add('docProps/core.xml', `${XML}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>${xmlText(deck.title)}</dc:title><dc:creator>${xmlText(deck.author ?? 'ArcLang')}</dc:creator></cp:coreProperties>`);
  add('docProps/app.xml', `${XML}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>ArcLang</Application><Slides>${count}</Slides><Notes>${count}</Notes></Properties>`);
  add('ppt/presentation.xml', presentation(count));
  add('ppt/_rels/presentation.xml.rels', rels([
    ['rId1', 'slideMaster', 'slideMasters/slideMaster1.xml'],
    ['rId2', 'notesMaster', 'notesMasters/notesMaster1.xml'],
    ['rId3', 'theme', 'theme/theme1.xml'],
    ['rId4', 'presProps', 'presProps.xml'],
    ['rId5', 'viewProps', 'viewProps.xml'],
    ['rId6', 'tableStyles', 'tableStyles.xml'],
    ...deck.slides.map((_, i) => [`rId${10 + i}`, 'slide', `slides/slide${i + 1}.xml`] as [string, string, string]),
  ]));
  add('ppt/presProps.xml', `${XML}<p:presentationPr xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}"/>`);
  add('ppt/viewProps.xml', `${XML}<p:viewPr xmlns:a="${NS_A}" xmlns:r="${NS_R}" xmlns:p="${NS_P}"><p:normalViewPr><p:restoredLeft sz="15620"/><p:restoredTop sz="80000"/></p:normalViewPr><p:gridSpacing cx="76200" cy="76200"/></p:viewPr>`);
  add('ppt/tableStyles.xml', `${XML}<a:tblStyleLst xmlns:a="${NS_A}" def="{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}"/>`);
  add('ppt/slideMasters/slideMaster1.xml', master());
  add('ppt/slideMasters/_rels/slideMaster1.xml.rels', rels([['rId1', 'slideLayout', '../slideLayouts/slideLayout1.xml'], ['rId2', 'theme', '../theme/theme1.xml']]));
  add('ppt/slideLayouts/slideLayout1.xml', layout());
  add('ppt/slideLayouts/_rels/slideLayout1.xml.rels', rels([['rId1', 'slideMaster', '../slideMasters/slideMaster1.xml']]));
  add('ppt/theme/theme1.xml', theme('ArcLang'));
  add('ppt/theme/theme2.xml', theme('ArcLang notes'));
  add('ppt/notesMasters/notesMaster1.xml', notesMaster());
  add('ppt/notesMasters/_rels/notesMaster1.xml.rels', rels([['rId1', 'theme', '../theme/theme2.xml']]));
  deck.slides.forEach((item, i) => {
    if (!isPng(item.png)) throw new Error(`slide ${i + 1} (${item.title}): the picture is not a PNG`);
    const n = i + 1;
    add(`ppt/slides/slide${n}.xml`, slide(item, i));
    add(`ppt/slides/_rels/slide${n}.xml.rels`, rels([
      ['rId1', 'slideLayout', '../slideLayouts/slideLayout1.xml'],
      ['rId2', 'image', `../media/image${n}.png`],
      ['rId3', 'notesSlide', `../notesSlides/notesSlide${n}.xml`],
    ]));
    add(`ppt/notesSlides/notesSlide${n}.xml`, notesSlide(item));
    add(`ppt/notesSlides/_rels/notesSlide${n}.xml.rels`, rels([['rId1', 'notesMaster', '../notesMasters/notesMaster1.xml'], ['rId2', 'slide', `../slides/slide${n}.xml`]]));
    add(`ppt/media/image${n}.png`, item.png);
  });
  return zip(entries, deflate);
}

/** The width and height a PNG declares (its IHDR), or null. */
export function pngSize(data: Uint8Array): { width: number; height: number } | null {
  if (!isPng(data) || data.length < 24) return null;
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return { width: view.getUint32(16), height: view.getUint32(20) };
}
