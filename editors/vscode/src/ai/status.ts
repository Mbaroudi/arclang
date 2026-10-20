// The status bar item of the AI: the provider and the model, a click opens
// the quick configuration. Shown beside a model (text or workbench).
import * as vscode from 'vscode';
import { broadcastStatus } from './bridge';
import { currentConfig } from './settings';
import { describeConfig, missingPiece } from './providers';

export function registerAiStatus(context: vscode.ExtensionContext): void {
  const item = vscode.window.createStatusBarItem('arclang.ai', vscode.StatusBarAlignment.Right, 99);
  item.name = 'ArcLang AI';
  item.command = 'arclang.ai.configure';
  const refresh = async () => {
    const config = await currentConfig(context);
    const problem = missingPiece(config);
    item.text = config.provider === 'off' ? '$(circle-slash) AI off' : `$(sparkle) ${describeConfig(config).replace('VS Code language models (Copilot…)', 'VS Code LM')}`;
    item.tooltip = new vscode.MarkdownString(`**ArcLang AI**: ${describeConfig(config)}${config.endpoint && config.provider !== 'vscode-lm' ? `\n\nEndpoint: \`${config.endpoint}\`` : ''}${problem ? `\n\n$(warning) ${problem}` : ''}\n\nClick to configure.`, true);
    item.backgroundColor = problem && config.provider !== 'off' ? new vscode.ThemeColor('statusBarItem.warningBackground') : undefined;
    const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input;
    const onModel = (vscode.window.activeTextEditor && ['arclang', 'sysml'].includes(vscode.window.activeTextEditor.document.languageId)) || (tab instanceof vscode.TabInputCustom && tab.viewType === 'arclang.workbench');
    if (onModel) item.show();
    else item.hide();
  };
  context.subscriptions.push(
    item,
    vscode.window.onDidChangeActiveTextEditor(() => void refresh()),
    vscode.window.tabGroups.onDidChangeTabs(() => void refresh()),
    vscode.window.tabGroups.onDidChangeTabGroups(() => void refresh()),
    context.secrets.onDidChange(() => {
      void refresh();
      void broadcastStatus(context);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('arclang.ai')) {
        void refresh();
        void broadcastStatus(context);
      }
    }),
  );
  void refresh();
}
