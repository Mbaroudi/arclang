import { defineConfig } from '@vscode/test-cli';

export default defineConfig({
  files: 'out/test/suite/**/*.test.js',
  workspaceFolder: '../../examples',
  launchArgs: ['--disable-extensions'],
  mocha: { ui: 'tdd', timeout: 120000 },
});
