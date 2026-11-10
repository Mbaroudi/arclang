import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mcpEnvironment } from '../../src/pure/mcp-env';

test('the server works in the folders that are open, the first being its workspace', () => {
  assert.deepEqual(mcpEnvironment({ folders: ['/work/ev'], binary: '/opt/arclang', inherited: {}, delimiter: ':' }), {
    ARCLANG_BINARY: '/opt/arclang',
    ARCLANG_WORKSPACE: '/work/ev',
    ARCLANG_MCP_ROOTS: '/work/ev',
  });
  assert.deepEqual(mcpEnvironment({ folders: ['/work/ev', '/work/library'], binary: null, inherited: {}, delimiter: ':' }), {
    ARCLANG_WORKSPACE: '/work/ev',
    ARCLANG_MCP_ROOTS: '/work/ev:/work/library',
  });
  assert.equal(mcpEnvironment({ folders: ['C:\\work\\ev', 'D:\\library'], binary: null, inherited: {}, delimiter: ';' }).ARCLANG_MCP_ROOTS, 'C:\\work\\ev;D:\\library');
});

test('without a folder there is no workspace to keep to: any path, as a file opened alone needs', () => {
  assert.deepEqual(mcpEnvironment({ folders: [], binary: null, inherited: {}, delimiter: ':' }), { ARCLANG_MCP_ROOTS: '*' });
});

test('what the user set in the environment is theirs', () => {
  const inherited = { ARCLANG_WORKSPACE: '/models', ARCLANG_MCP_ROOTS: '/models:/exports' };
  assert.deepEqual(mcpEnvironment({ folders: ['/work/ev'], binary: null, inherited, delimiter: ':' }), {});
  // A workspace of their own, and the open folders still reachable.
  assert.deepEqual(mcpEnvironment({ folders: ['/work/ev'], binary: null, inherited: { ARCLANG_WORKSPACE: '/models' }, delimiter: ':' }), { ARCLANG_MCP_ROOTS: '/work/ev' });
  // An empty variable is not a choice.
  assert.equal(mcpEnvironment({ folders: ['/work/ev'], binary: null, inherited: { ARCLANG_WORKSPACE: '' }, delimiter: ':' }).ARCLANG_WORKSPACE, '/work/ev');
});

test('a folder whose path holds the separator cannot be named among the others', () => {
  assert.deepEqual(mcpEnvironment({ folders: ['/work/ev', '/work/a:b'], binary: null, inherited: {}, delimiter: ':' }), {
    ARCLANG_WORKSPACE: '/work/ev',
    ARCLANG_MCP_ROOTS: '/work/ev',
  });
  // Alone, it is the workspace: the server keeps to it without the list.
  assert.deepEqual(mcpEnvironment({ folders: ['/work/a:b'], binary: null, inherited: {}, delimiter: ':' }), { ARCLANG_WORKSPACE: '/work/a:b' });
});
