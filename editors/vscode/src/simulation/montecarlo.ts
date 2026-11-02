// *ArcLang: Show Monte Carlo results*: the cases of a model that sample their
// uncertain parameters, drawn on one page: the probability the samples show
// against the one required, the spread of each margin, what moves it, the
// failing samples with a button to replay each on the diagram, and the
// further analysis of R or Python (`arclang verify --statistics`). The
// outcomes are those of the run just made, or of `arclang verify --json`,
// which answers from its cache when the model has not changed.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from '../cli';
import { MonteCarloCase, monteCarloCases, parseStatistics } from '../pure/montecarlo';
import { StatisticsState, monteCarloHtml, statisticsBlock } from '../pure/montecarlo-html';
import { Outcome } from '../pure/simulation';
import { replayCase } from './replay';

export class MonteCarloPanel {
  private static current: MonteCarloPanel | null = null;

  private cases: MonteCarloCase[] = [];
  private statistics: Record<string, StatisticsState | undefined> = {};
  private stale = false;
  private readonly disposables: vscode.Disposable[] = [];

  private constructor(private readonly panel: vscode.WebviewPanel, private file: string, private readonly env: () => Record<string, string>) {
    panel.onDidDispose(() => this.dispose(), null, this.disposables);
    panel.webview.onDidReceiveMessage((message: unknown) => void this.fromPage(message), null, this.disposables);
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.uri.fsPath === this.file && !this.stale) {
        this.stale = true;
        this.paint();
      }
    }, null, this.disposables);
  }

  /** Whether a run has Monte Carlo cases to show. */
  static has(outcomes: Outcome[]): boolean {
    return outcomes.some((o) => o.probability !== null);
  }

  /** Show the Monte Carlo cases of a run of `file`; `reveal: false` only refreshes a page already open. */
  static show(file: string, outcomes: Outcome[], env: () => Record<string, string>, reveal = true): void {
    let page = MonteCarloPanel.current;
    if (!page) {
      if (!reveal) return;
      const panel = vscode.window.createWebviewPanel('arclang.monteCarlo', 'Monte Carlo results', vscode.ViewColumn.Beside, { enableScripts: true, retainContextWhenHidden: true });
      page = new MonteCarloPanel(panel, file, env);
      MonteCarloPanel.current = page;
    }
    if (page.file !== file) page.statistics = {};
    page.file = file;
    page.cases = monteCarloCases(outcomes);
    page.stale = false;
    page.panel.title = `Monte Carlo: ${path.basename(file)}`;
    page.paint();
    if (reveal) page.panel.reveal(undefined, true);
  }

  private paint(): void {
    const nonce = randomBytes(16).toString('base64').replace(/[^A-Za-z0-9]/g, '');
    this.panel.webview.html = monteCarloHtml({ file: vscode.workspace.asRelativePath(this.file), cases: this.cases, statistics: this.statistics, stale: this.stale }, nonce, this.panel.webview.cspSource);
  }

  private async fromPage(message: unknown): Promise<void> {
    if (typeof message !== 'object' || message === null) return;
    const { type, testCase, index, engine } = message as { type?: unknown; testCase?: unknown; index?: unknown; engine?: unknown };
    // Only a case of the page, found by its id; only a sample it has; only an engine named here.
    const found = this.cases.find((c) => c.id === testCase);
    if (!found) return;
    if (type === 'replay' && typeof index === 'number' && Number.isInteger(index)) {
      const total = found.estimate?.total ?? Number.MAX_SAFE_INTEGER;
      if (index >= total) return;
      return replayCase(vscode.Uri.file(this.file), { id: found.id, name: found.name }, index >= 0 ? index : undefined);
    }
    if (type === 'statistics' && (engine === 'r' || engine === 'python')) return this.analyze(found, engine);
  }

  private async analyze(found: MonteCarloCase, engine: 'r' | 'python'): Promise<void> {
    if (this.statistics[found.id]?.state === 'running') return;
    this.setStatistics(found, { state: 'running', engine });
    const result = await run(['verify', this.file, '--test', found.id, '--statistics', engine, '--json'], { cwd: path.dirname(this.file), env: this.env() });
    const report = result.code === 0 ? parseStatistics(result.stdout) : null;
    const state: StatisticsState = report ? { state: 'done', report } : { state: 'error', message: failureText(result).split('\n').filter((l) => l.trim()).pop() ?? 'the analysis failed' };
    this.setStatistics(found, state);
  }

  /** Keep the state for the next full paint; send only its block to the page. */
  private setStatistics(found: MonteCarloCase, state: StatisticsState): void {
    this.statistics = { ...this.statistics, [found.id]: state };
    void this.panel.webview.postMessage({ type: 'statistics', testCase: found.id, html: statisticsBlock(found, state) });
  }

  private dispose(): void {
    MonteCarloPanel.current = null;
    while (this.disposables.length) this.disposables.pop()?.dispose();
  }
}
