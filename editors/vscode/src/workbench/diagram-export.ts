// The host side of a workbench's diagram exports and color themes: the
// themes sent to the page and the one it draws, a theme saved for the team
// or the user, an image or an SVG put on the system clipboard, a PowerPoint
// file written from the slides the page drew.
import * as path from 'node:path';
import { deflateRawSync } from 'node:zlib';
import * as vscode from 'vscode';
import { buildPptx, pngSize } from '../pure/pptx';
import type { DiagramAction, FromWebview, ThemeEntry, ToWebview } from '../pure/workbench';
import { writeImageToClipboard } from './clipboard-image';
import { ThemeStore } from './themes';

let store: ThemeStore | null = null;

/** The themes of every workbench; created once by the extension. */
export function themeStore(context: vscode.ExtensionContext): ThemeStore {
  if (!store) {
    store = new ThemeStore(context);
    context.subscriptions.push(store);
  }
  return store;
}

/** What a command passes to skip the questions (the tests, a script). */
export interface ExportArgs {
  scope?: 'current' | 'layer' | 'all';
  target?: string;
}

const isDark = () => [vscode.ColorThemeKind.Dark, vscode.ColorThemeKind.HighContrast].includes(vscode.window.activeColorTheme.kind);

export class DiagramExports implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private pending: ExportArgs = {};

  constructor(
    private readonly store: ThemeStore,
    private readonly document: vscode.TextDocument,
    private readonly post: (message: ToWebview) => void,
  ) {
    this.disposables.push(
      store.onDidChange(() => void this.pushThemes()),
      vscode.window.onDidChangeActiveColorTheme(() => void this.pushThemes()),
    );
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
  }

  /** The themes on offer and the one to draw, to the page. */
  async pushThemes(): Promise<void> {
    const entries = await this.store.list();
    const { entry, problem } = await this.store.active(entries);
    const listed = entries.some((e) => e.key === entry.key) ? entries : [...entries, entry];
    const copyBackground = vscode.workspace.getConfiguration().get<string>('arclang.diagram.copyBackground') === 'transparent' ? 'transparent' : 'white';
    this.post({ type: 'themes', entries: listed, active: entry.key, problem, dark: isDark(), copyBackground });
    if (problem) void vscode.window.showWarningMessage(problem);
  }

  /** A diagram command: the page does the drawing. */
  run(action: DiagramAction, args?: ExportArgs): void {
    this.pending = args ?? {};
    this.post({ type: 'diagram-action', action });
  }

  /** A message of the page about exports or themes; false when it is not one. */
  async handle(message: FromWebview): Promise<boolean> {
    switch (message.type) {
      case 'clipboard-image':
        await this.copyImage(message.data, message.cropped);
        return true;
      case 'clipboard-svg':
        await vscode.env.clipboard.writeText(message.text);
        vscode.window.setStatusBarMessage('ArcLang: the diagram is on the clipboard as SVG', 4000);
        return true;
      case 'notify':
        if (message.error) void vscode.window.showErrorMessage(`ArcLang: ${message.text}`);
        else vscode.window.setStatusBarMessage(`ArcLang: ${message.text}`, 4000);
        return true;
      case 'pptx-request':
        await this.askSlides(message.views, message.current);
        return true;
      case 'pptx':
        await this.writePptx(message.slides, message.title, message.target);
        return true;
      case 'theme-choose': {
        const entry = (await this.store.list()).find((e) => e.key === message.key);
        if (!entry) return true;
        if (!entry.ok) {
          void vscode.window.showErrorMessage(`ArcLang: the theme "${entry.name}" is refused: ${entry.errors.join('; ')}`);
          return true;
        }
        await this.store.choose(entry);
        return true;
      }
      case 'theme-save':
        await this.saveTheme(message.scope, message.theme);
        return true;
      case 'theme-export': {
        const target = await this.store.exportFile(message.theme);
        if (target) vscode.window.setStatusBarMessage(`ArcLang: theme exported to ${path.basename(target.fsPath)}`, 4000);
        return true;
      }
      case 'theme-import': {
        try {
          const read = await this.store.importFile();
          if (read) this.post({ type: 'theme-imported', theme: read.theme, errors: read.verdict.errors.map((e) => e.message), name: read.name });
        } catch (error) {
          void vscode.window.showErrorMessage(`ArcLang: the theme could not be imported: ${error instanceof Error ? error.message : String(error)}`);
        }
        return true;
      }
      default:
        return false;
    }
  }

  private async copyImage(base64: string, cropped: boolean): Promise<void> {
    try {
      await writeImageToClipboard(Buffer.from(base64, 'base64'));
      vscode.window.setStatusBarMessage(`ArcLang: ${cropped ? 'the selection' : 'the diagram'} is on the clipboard as an image: paste it in a slide or a document`, 5000);
    } catch (error) {
      void vscode.window.showErrorMessage(`ArcLang: the image could not be put on the clipboard: ${error instanceof Error ? error.message : String(error)}. Save it as PNG from Export instead.`);
    }
  }

  private async saveTheme(scope: 'team' | 'user', theme: Record<string, unknown>): Promise<void> {
    try {
      const entry = await this.store.save(scope, theme);
      await this.store.choose(entry);
      this.post({ type: 'theme-saved', entry });
      vscode.window.setStatusBarMessage(`ArcLang: theme "${entry.name}" saved ${scope === 'team' ? `for the team in ${vscode.workspace.asRelativePath(entry.path ?? '')}` : 'in your themes'}`, 5000);
    } catch (error) {
      void vscode.window.showErrorMessage(`ArcLang: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private async askSlides(views: { id: string; title: string; view: string }[], current: string | null): Promise<void> {
    const here = views.find((v) => v.id === current) ?? views[0];
    if (!here) return;
    const layer = views.filter((v) => v.view === here.view);
    let scope = this.pending.scope;
    if (!scope) {
      const items: (vscode.QuickPickItem & { scope: 'current' | 'layer' | 'all' })[] = [
        { label: 'This diagram', description: here.title, scope: 'current' },
        { label: 'Its layer', description: `${layer.length} diagram${layer.length > 1 ? 's' : ''}`, scope: 'layer' },
        { label: 'Every diagram', description: `${views.length} diagrams, one slide each`, scope: 'all' },
      ];
      const picked = await vscode.window.showQuickPick(items, { placeHolder: 'Which diagrams go into the PowerPoint file?' });
      if (!picked) return;
      scope = picked.scope;
    }
    const ids = scope === 'current' ? [here.id] : scope === 'layer' ? layer.map((v) => v.id) : views.map((v) => v.id);
    this.post({ type: 'pptx-render', ids, target: this.pending.target });
  }

  private async writePptx(slides: { title: string; data: string; width: number; height: number; notes: string[] }[], title: string, target?: string): Promise<void> {
    try {
      const deck = buildPptx(
        {
          title,
          slides: slides.map((slide) => {
            const png = new Uint8Array(Buffer.from(slide.data, 'base64'));
            const size = pngSize(png);
            return { title: slide.title, png, width: size?.width ?? slide.width, height: size?.height ?? slide.height, notes: slide.notes };
          }),
        },
        (data) => new Uint8Array(deflateRawSync(data)),
      );
      const folder = this.document.uri.scheme === 'file' ? vscode.Uri.file(path.dirname(this.document.uri.fsPath)) : undefined;
      const name = `${path.basename(this.document.uri.fsPath).replace(/\.[^.]+$/, '') || 'diagrams'}.pptx`;
      const uri = target
        ? vscode.Uri.file(target)
        : await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.joinPath(folder, name) : undefined, filters: { 'PowerPoint presentation': ['pptx'] }, saveLabel: 'Export' });
      if (!uri) return;
      await vscode.workspace.fs.writeFile(uri, deck);
      if (target) {
        vscode.window.setStatusBarMessage(`ArcLang: ${slides.length} slide${slides.length > 1 ? 's' : ''} written to ${path.basename(uri.fsPath)}`, 5000);
        return;
      }
      const open = await vscode.window.showInformationMessage(`ArcLang: ${slides.length} slide${slides.length > 1 ? 's' : ''} written to ${path.basename(uri.fsPath)}`, 'Reveal');
      if (open === 'Reveal') await vscode.commands.executeCommand('revealFileInOS', uri);
    } catch (error) {
      void vscode.window.showErrorMessage(`ArcLang: the PowerPoint file could not be written: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      this.pending = {};
    }
  }
}

/** The entry of `entries` the theme picker shows first: the active one. */
export function pickerItems(entries: ThemeEntry[], active: string): (vscode.QuickPickItem & { entry: ThemeEntry })[] {
  const group: Record<string, string> = { preset: 'Built-in', team: 'Team (.arclang/themes)', user: 'Yours', file: 'File' };
  return entries.map((entry) => ({
    label: `${entry.key === active ? '$(check) ' : ''}${entry.name}`,
    description: entry.ok ? group[entry.source] : `${group[entry.source]} — refused`,
    detail: entry.ok ? entry.description : entry.errors.join('; '),
    entry,
  }));
}
