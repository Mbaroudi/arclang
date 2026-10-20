// The theme editor: a panel over the canvas where a theme is made from a
// copy of another one, a colour per concept and per exchange, previewed on
// the diagram as it is picked and checked by the same rules as everywhere
// (ArcVizTheme). A theme that breaks a rule can be previewed, not saved.
import type { ThemeEntry, ToWebview } from '../src/pure/workbench';
import { button, clear, el } from './dom';
import type { ViewerApi } from './globals';

interface ThemeModule {
  CONCEPTS: { key: string; label: string }[];
  EXCHANGES: { key: string; label: string }[];
  validate(theme: unknown): { ok: boolean; errors: { message: string }[]; warnings: { message: string }[]; theme: Record<string, unknown> | null };
  complete(theme: unknown): Record<string, unknown>;
  duplicate(theme: unknown, name: string): Record<string, unknown>;
}

export interface EditorDeps {
  host: HTMLElement;
  post(message: unknown): void;
  viewer(): ViewerApi | null;
  entries(): ThemeEntry[];
  active(): ThemeEntry | null;
  /** Whether a preview should draw the theme's dark variant. */
  dark(entry: Record<string, unknown>): boolean;
  /** Put the theme in use back on the diagram. */
  restore(): Promise<void>;
}

type Draft = Record<string, unknown> & { concepts: Record<string, Record<string, string>>; exchanges: Record<string, Record<string, string | number>> };

const HEX = /^#[0-9a-fA-F]{6}$/;
const cap = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);

export class ThemeEditor {
  private readonly root: HTMLElement;
  private draft: Draft | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly verdictBox: HTMLElement;
  private readonly fields: HTMLElement;
  private readonly base: HTMLSelectElement;
  private readonly name: HTMLInputElement;
  private readonly saveTeam: HTMLButtonElement;
  private readonly saveMine: HTMLButtonElement;
  private readonly exportJson: HTMLButtonElement;

  constructor(private readonly d: EditorDeps) {
    this.base = el('select', { class: 'wb-te-base', 'aria-label': 'Theme to start from' });
    this.name = el('input', { type: 'text', class: 'wb-te-name', 'aria-label': 'Name of the new theme', maxlength: '80', placeholder: 'Name of the theme' });
    this.name.addEventListener('input', () => this.changed());
    this.verdictBox = el('div', { class: 'wb-te-verdict', role: 'status', 'aria-live': 'polite' });
    this.fields = el('div', { class: 'wb-te-fields' });
    this.saveTeam = button('Save as team theme', () => this.save('team'), { class: 'wb-primary', title: 'Into .arclang/themes of the workspace: versioned with the model, the same for everyone' });
    this.saveMine = button('Save as my theme', () => this.save('user'), { title: 'Into your own themes, for every model you open' });
    this.exportJson = button('Export JSON…', () => this.draft && this.d.post({ type: 'theme-export', theme: this.clean() }));
    const close = button('×', () => void this.close(), { class: 'wb-te-close', 'aria-label': 'Close the theme editor', title: 'Close (Esc): the theme in use comes back' });
    this.root = el('section', { class: 'wb-theme-editor', role: 'dialog', 'aria-label': 'Colour theme', hidden: true }, [
      el('header', { class: 'wb-te-head' }, [el('h2', { text: 'Colour theme' }), close]),
      el('div', { class: 'wb-te-start' }, [
        el('label', { text: 'Start from' }),
        this.base,
        button('Duplicate', () => this.duplicate(), { title: 'A copy of this theme, to edit under a name of its own' }),
        button('Import JSON…', () => this.d.post({ type: 'theme-import' }), { title: 'An arclang-theme.json file: checked before anything is drawn' }),
      ]),
      el('div', { class: 'wb-te-row' }, [el('label', { text: 'Name' }), this.name]),
      this.verdictBox,
      this.fields,
      el('footer', { class: 'wb-te-actions' }, [this.saveTeam, this.saveMine, this.exportJson]),
    ]);
    this.root.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        void this.close();
      }
    });
    d.host.appendChild(this.root);
  }

  private get module(): ThemeModule {
    return (window as unknown as { ArcVizTheme: ThemeModule }).ArcVizTheme;
  }

  get isOpen(): boolean {
    return !this.root.hidden;
  }

  open(): void {
    clear(this.base);
    const entries = this.d.entries().filter((e) => e.ok && e.theme);
    const active = this.d.active();
    for (const entry of entries) {
      const group = entry.source === 'preset' ? 'built-in' : entry.source === 'team' ? 'team' : entry.source === 'user' ? 'yours' : 'file';
      this.base.appendChild(el('option', { value: entry.key, text: `${entry.name} (${group})`, selected: entry.key === active?.key }));
    }
    this.root.hidden = false;
    if (!this.draft) this.duplicate();
    else this.changed();
    this.base.focus();
  }

  async close(): Promise<void> {
    if (this.root.hidden) return;
    this.root.hidden = true;
    clearTimeout(this.timer);
    await this.d.restore();
  }

  /** The host answered: a theme saved (kept as the draft's new base), or a file read. */
  handle(message: ToWebview): boolean {
    if (message.type === 'theme-saved') {
      this.verdictBox.prepend(el('p', { class: 'wb-te-saved', text: `Saved: ${message.entry.name}, now the theme of every diagram.` }));
      return true;
    }
    if (message.type === 'theme-imported') {
      if (!this.isOpen) this.open();
      if (message.errors.length && !(message.theme && typeof message.theme === 'object')) {
        this.showVerdict({ ok: false, errors: message.errors.map((m) => ({ message: m })), warnings: [] }, `${message.name} is not a theme`);
        return true;
      }
      const theme = message.theme as Record<string, unknown>;
      try {
        this.draft = this.module.complete(theme) as Draft;
      } catch {
        this.draft = null;
      }
      if (!this.draft || !this.draft.concepts) {
        this.showVerdict({ ok: false, errors: message.errors.map((m) => ({ message: m })), warnings: [] }, `${message.name} is refused`);
        return true;
      }
      this.name.value = String(this.draft.name ?? message.name);
      this.drawFields();
      this.changed();
      return true;
    }
    return false;
  }

  private duplicate(): void {
    const entry = this.d.entries().find((e) => e.key === this.base.value) ?? this.d.active();
    if (!entry?.theme) return;
    this.draft = this.module.duplicate(entry.theme, `${entry.name} (copy)`) as Draft;
    this.name.value = String(this.draft.name);
    this.drawFields();
    this.changed();
  }

  private colour(label: string, get: () => string, set: (value: string) => void): HTMLElement {
    const swatch = el('input', { type: 'color', class: 'wb-te-swatch', value: get().toLowerCase(), 'aria-label': `${label} colour` });
    const hex = el('input', { type: 'text', class: 'wb-te-hex', value: get().toUpperCase(), 'aria-label': `${label} hex`, maxlength: '7', spellcheck: 'false' });
    swatch.addEventListener('input', () => {
      hex.value = swatch.value.toUpperCase();
      set(hex.value);
      this.changed();
    });
    hex.addEventListener('input', () => {
      const value = hex.value.trim();
      hex.classList.toggle('is-invalid', !HEX.test(value));
      if (!HEX.test(value)) return;
      swatch.value = value.toLowerCase();
      set(value.toUpperCase());
      this.changed();
    });
    return el('span', { class: 'wb-te-colour' }, [swatch, hex]);
  }

  private drawFields(): void {
    const draft = this.draft;
    clear(this.fields);
    if (!draft) return;
    const sheet = el('fieldset', { class: 'wb-te-group' }, [el('legend', { text: 'Sheet and text' })]);
    for (const [key, label] of [['background', 'Background'], ['text', 'Text'], ['muted', 'Secondary text']] as const) {
      sheet.appendChild(el('div', { class: 'wb-te-line', 'data-key': key }, [el('span', { class: 'wb-te-label', text: label }), this.colour(label, () => String(draft[key]), (v) => { draft[key] = v; })]));
    }
    const concepts = el('fieldset', { class: 'wb-te-group' }, [el('legend', { text: 'Concepts: fill and outline' })]);
    for (const concept of this.module.CONCEPTS) {
      const entry = draft.concepts[concept.key];
      concepts.appendChild(el('div', { class: 'wb-te-line', 'data-key': concept.key }, [
        el('span', { class: 'wb-te-label', text: cap(concept.label) }),
        this.colour(`${concept.label} fill`, () => entry.fill, (v) => { entry.fill = v; }),
        this.colour(`${concept.label} outline`, () => entry.stroke, (v) => { entry.stroke = v; }),
      ]));
    }
    const exchanges = el('fieldset', { class: 'wb-te-group' }, [el('legend', { text: 'Exchanges: line and style' })]);
    for (const exchange of this.module.EXCHANGES) {
      const entry = draft.exchanges[exchange.key];
      const style = el('select', { class: 'wb-te-style', 'aria-label': `${exchange.label} style` });
      for (const option of ['solid', 'dashed', 'dotted']) style.appendChild(el('option', { value: option, text: option, selected: (entry.style ?? 'solid') === option }));
      style.addEventListener('change', () => { entry.style = style.value; this.changed(); });
      exchanges.appendChild(el('div', { class: 'wb-te-line', 'data-key': exchange.key }, [
        el('span', { class: 'wb-te-label', text: cap(exchange.label) }),
        this.colour(`${exchange.label} line`, () => String(entry.line), (v) => { entry.line = v; }),
        style,
      ]));
    }
    this.fields.append(sheet, concepts, exchanges);
  }

  /** The draft as it would be saved: its name, no source, the version. */
  private clean(): Record<string, unknown> {
    const out = JSON.parse(JSON.stringify(this.draft)) as Record<string, unknown>;
    out.name = this.name.value.trim() || 'Untitled theme';
    out.version = 1;
    delete out.source;
    delete out.id;
    return out;
  }

  private changed(): void {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.check(), 120);
  }

  private async check(): Promise<void> {
    if (!this.draft) return;
    const theme = this.clean();
    const verdict = this.module.validate(theme);
    this.showVerdict(verdict, null);
    // The preview: the draft on the diagram, whatever it breaks; never kept.
    await this.d.viewer()?.setTheme?.(theme, { draft: true, dark: this.d.dark(theme) });
  }

  private showVerdict(verdict: { ok: boolean; errors: { message: string }[]; warnings: { message: string }[] }, heading: string | null): void {
    clear(this.verdictBox);
    this.verdictBox.classList.toggle('is-refused', !verdict.ok);
    const title = heading ?? (verdict.ok ? 'Passes every colour rule' : `Refused: ${verdict.errors.length} rule${verdict.errors.length > 1 ? 's' : ''} broken — previewed, cannot be saved`);
    this.verdictBox.appendChild(el('p', { class: 'wb-te-title', text: `${verdict.ok ? '✓' : '✕'} ${title}` }));
    if (verdict.errors.length) this.verdictBox.appendChild(el('ul', { class: 'wb-te-errors' }, verdict.errors.map((e) => el('li', { text: e.message }))));
    if (verdict.warnings.length) {
      const details = el('details', { class: 'wb-te-warnings' }, [el('summary', { text: `${verdict.warnings.length} warning${verdict.warnings.length > 1 ? 's' : ''}` }), el('ul', {}, verdict.warnings.map((w) => el('li', { text: w.message })))]);
      this.verdictBox.appendChild(details);
    }
    for (const control of [this.saveTeam, this.saveMine, this.exportJson]) control.disabled = !verdict.ok;
  }

  private save(scope: 'team' | 'user'): void {
    if (!this.draft) return;
    const theme = this.clean();
    if (!this.module.validate(theme).ok) return;
    this.d.post({ type: 'theme-save', scope, theme });
  }
}
