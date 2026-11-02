// The page side of the diagram exports and color themes: the theme drawn
// (its dark variant on a dark editor when it has one, else the editor's
// inversion), Copy as image and Copy as SVG of the view or the selection,
// the slides of a PowerPoint export, and the entries these add to the
// toolbar's Export and Theme menus and to the canvas menu.
import type { DiagramAction, SlideData, ThemeEntry, ToWebview } from '../src/pure/workbench';
import type { ToolbarHookItem, ViewerApi } from './globals';
import type { MenuItem } from './menu';

export interface ExportDeps {
  post(message: unknown): void;
  viewer(): ViewerApi | null;
  /** The drawing ids of the selection (one box, or the multi-selection). */
  selection(): string[];
  toast(text: string, kind: 'info' | 'error'): void;
  /** The model's file name, for the PowerPoint title. */
  file(): string;
  openEditor(): void;
}

/** The long side of a slide's picture, in pixels: sharp on a 4K projector. */
const SLIDE_PIXELS = 2400;
const VIEW_NAMES: Record<string, string> = { oab: 'Operational', sab: 'System', lab: 'Logical', pab: 'Physical', msm: 'Modes and states', es: 'Scenario', cap: 'Capabilities', cdb: 'Data', pbs: 'Product breakdown' };

function base64Of(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const url = String(reader.result);
      resolve(url.slice(url.indexOf(',') + 1));
    };
    reader.onerror = () => reject(reader.error ?? new Error('the image could not be read'));
    reader.readAsDataURL(blob);
  });
}

export class DiagramExports {
  entries: ThemeEntry[] = [];
  active: string | null = null;
  private dark = false;
  private copyBackground: 'white' | 'transparent' = 'white';
  private applied: string | null = null;

  constructor(private readonly d: ExportDeps) {}

  /** The active theme's entry. */
  activeEntry(): ThemeEntry | null {
    return this.entries.find((e) => e.key === this.active) ?? null;
  }

  /** Whether the dark editor shows the theme's own dark variant (else the inversion of its light one). */
  ownDark(entry = this.activeEntry()): boolean {
    return this.dark && !!entry?.theme && typeof entry.theme.dark === 'object' && entry.theme.dark !== null;
  }

  /** What the viewer is mounted with: the active theme, dark when it has a variant for a dark editor. */
  mountTheme(): { theme: Record<string, unknown>; dark: boolean } | undefined {
    const entry = this.activeEntry();
    this.applied = entry ? `${entry.key}|${this.ownDark(entry)}|${JSON.stringify(entry.theme)}` : null;
    this.markDark();
    return entry?.theme ? { theme: entry.theme, dark: this.ownDark(entry) } : undefined;
  }

  private markDark(): void {
    // The editor's inversion of a light sheet stops where the theme draws its own dark one.
    document.body.classList.toggle('wb-theme-own-dark', this.ownDark());
  }

  /** Draw the active theme again (after the editor's preview, a new theme chosen). */
  async reapply(): Promise<void> {
    const viewer = this.d.viewer();
    const entry = this.activeEntry();
    this.markDark();
    if (!viewer?.setTheme) return;
    await viewer.ready;
    const verdict = await viewer.setTheme(entry?.theme ?? null, { dark: this.ownDark(entry) });
    if (!verdict.ok) this.d.toast(`The theme ${entry?.name ?? ''} is refused: ${verdict.errors.map((e) => e.message).join('; ')}`, 'error');
    this.applied = entry ? `${entry.key}|${this.ownDark(entry)}|${JSON.stringify(entry.theme)}` : null;
  }

  /** A message of the host about themes or exports; false when it is not one. */
  handle(message: ToWebview): boolean {
    switch (message.type) {
      case 'themes': {
        this.entries = message.entries;
        this.active = message.active;
        this.dark = message.dark;
        this.copyBackground = message.copyBackground;
        const entry = this.activeEntry();
        const key = entry ? `${entry.key}|${this.ownDark(entry)}|${JSON.stringify(entry.theme)}` : null;
        if (key !== this.applied) void this.reapply();
        if (message.problem) this.d.toast(message.problem.replace(/^ArcLang: /, ''), 'error');
        return true;
      }
      case 'diagram-action':
        this.run(message.action);
        return true;
      case 'pptx-render':
        void this.renderSlides(message.ids, message.target);
        return true;
      default:
        return false;
    }
  }

  run(action: DiagramAction): void {
    if (action === 'copy-image') void this.copyImage();
    else if (action === 'copy-svg') void this.copySvg();
    else if (action === 'export-pptx') this.exportPptx();
    else if (action === 'edit-theme') this.d.openEditor();
    else if (action === 'choose-theme') this.d.openEditor();
  }

  /** The view, or the box around the selection, as a 2x PNG on the system clipboard. */
  async copyImage(): Promise<void> {
    const viewer = this.d.viewer();
    if (!viewer?.exportImage) return;
    try {
      const image = await viewer.exportImage({ ids: this.d.selection(), scale: 2, transparent: this.copyBackground === 'transparent' });
      if (!image) return;
      const what = image.cropped ? 'The selection' : 'The diagram';
      try {
        // The browser's clipboard first; VS Code may refuse it to a webview.
        if (typeof ClipboardItem === 'undefined' || !navigator.clipboard?.write) throw new Error('no clipboard for images here');
        await navigator.clipboard.write([new ClipboardItem({ 'image/png': image.blob })]);
        this.d.post({ type: 'notify', text: `${what.toLowerCase()} is on the clipboard as an image (${Math.round(image.width * 2)} × ${Math.round(image.height * 2)} px)` });
      } catch {
        // The host writes it natively (osascript, PowerShell, wl-copy or xclip).
        this.d.post({ type: 'clipboard-image', data: await base64Of(image.blob), width: Math.round(image.width * 2), height: Math.round(image.height * 2), cropped: image.cropped });
      }
    } catch (error) {
      this.d.toast(`The image could not be made: ${error instanceof Error ? error.message : String(error)}`, 'error');
    }
  }

  /** The SVG markup of the view or the selection, as text on the clipboard. */
  async copySvg(): Promise<void> {
    const drawing = await this.d.viewer()?.exportSvg?.({ ids: this.d.selection(), transparent: this.copyBackground === 'transparent' });
    if (drawing) this.d.post({ type: 'clipboard-svg', text: drawing.svg });
  }

  /** Ask the host which diagrams: it answers with pptx-render. */
  exportPptx(): void {
    const viewer = this.d.viewer();
    if (!viewer?.views) return;
    this.d.post({ type: 'pptx-request', views: viewer.views(), current: viewer.current?.().view ?? null });
  }

  private async renderSlides(ids: string[], target?: string): Promise<void> {
    const viewer = this.d.viewer();
    if (!viewer?.exportSvg || !viewer.exportImage) return;
    const slides: SlideData[] = [];
    this.d.toast(`Drawing ${ids.length} slide${ids.length > 1 ? 's' : ''}…`, 'info');
    try {
      for (const id of ids) {
        const svg = await viewer.exportSvg({ view: id });
        if (!svg) continue;
        const scale = Math.max(1, Math.min(4, SLIDE_PIXELS / Math.max(svg.width, svg.height)));
        const image = await viewer.exportImage({ view: id, scale });
        if (!image) continue;
        const kind = viewer.views?.().find((v) => v.id === id)?.view ?? '';
        const layer = VIEW_NAMES[kind];
        slides.push({
          title: layer && !image.title.toLowerCase().startsWith(layer.toLowerCase()) ? `${layer} — ${image.title}` : image.title,
          data: await base64Of(image.blob),
          width: Math.round(image.width * scale),
          height: Math.round(image.height * scale),
          notes: image.elements.map((e) => `${e.kind}: ${e.name}`),
        });
      }
    } catch (error) {
      this.d.toast(`The slides could not be drawn: ${error instanceof Error ? error.message : String(error)}`, 'error');
      return;
    }
    if (slides.length) this.d.post({ type: 'pptx', slides, title: this.d.file().replace(/\.[^.]+$/, ''), target });
  }

  /** More entries in the toolbar's Export menu. */
  exportItems(): ToolbarHookItem[] {
    return [
      { id: 'copy-image', label: 'Copy as image', title: 'The view, or the selection, as a picture to paste in PowerPoint, Keynote, Word or Google Slides', shortcut: 'Mod+Shift+C', run: () => void this.copyImage() },
      { id: 'copy-svg', label: 'Copy as SVG', title: 'The view, or the selection, as SVG text for the tools that take it', run: () => void this.copySvg() },
      { id: 'export-pptx', label: 'Export PowerPoint…', title: 'One 16:9 slide per diagram (this one, its layer or all), with its title and the elements in the notes', run: () => this.exportPptx() },
    ];
  }

  /** The Theme menu: every theme on offer (the one in use ticked), then the editor. */
  themeItems(): ToolbarHookItem[] {
    const choices = this.entries.map((entry) => ({
      id: `theme-${entry.key}`,
      label: entry.ok ? entry.name : `${entry.name} (refused)`,
      title: entry.ok ? (entry.description ?? `${entry.source} theme`) : entry.errors.join('; '),
      checked: entry.key === this.active,
      run: () => (entry.ok ? this.d.post({ type: 'theme-choose', key: entry.key }) : this.d.toast(`The theme ${entry.name} is refused: ${entry.errors.join('; ')}`, 'error')),
    }));
    return [...choices, { id: 'theme-edit', label: 'Edit or new theme…', title: 'Colors per concept, checked as you pick them, previewed on this diagram', run: () => this.d.openEditor() }];
  }

  /** The same, in the canvas menu. */
  menuItems(): MenuItem[] {
    return [
      { label: 'Copy as image', shortcut: '⇧⌘C', run: () => void this.copyImage() },
      { label: 'Copy as SVG', run: () => void this.copySvg() },
      { label: 'Export PowerPoint…', run: () => this.exportPptx() },
      { label: 'Theme…', run: () => this.d.openEditor() },
    ];
  }
}
