import assert from 'node:assert/strict';
import { test } from 'node:test';
import { importClosure, importsOf, resolveImport } from '../../src/pure/imports';

test('the imports of a text are the lines that import, not the ones that mention it', () => {
  const text = '// import "no.arc"\nimport "requirements.arc"\n  import "parts/logical.arc"\nmodel M { description: "import \\"x.arc\\"" }\n';
  assert.deepEqual(importsOf(text), ['requirements.arc', 'parts/logical.arc']);
});

test('an import is resolved beside the file that writes it', () => {
  assert.equal(resolveImport('main.arc', 'parts/a.arc'), 'parts/a.arc');
  assert.equal(resolveImport('parts/a.arc', 'b.arc'), 'parts/b.arc');
  assert.equal(resolveImport('parts/a.arc', '../shared/c.arc'), 'shared/c.arc');
  assert.equal(resolveImport('main.arc', '../shared/c.arc'), '../shared/c.arc');
  assert.equal(resolveImport('', './a.arc'), 'a.arc');
});

test('every file a model reaches is gathered once, a missing one left to the compiler to name', async () => {
  const disk: Record<string, string> = {
    'requirements.arc': 'requirements stakeholder {}\n',
    'parts/logical.arc': 'import "more.arc"\nimport "../main.arc"\n',
    'parts/more.arc': 'import "../requirements.arc"\nimport "gone.arc"\n',
  };
  const asked: string[] = [];
  const read = async (path: string) => {
    asked.push(path);
    return disk[path];
  };
  const files = await importClosure('main.arc', 'import "requirements.arc"\nimport "parts/logical.arc"\n', read);
  assert.deepEqual(Object.keys(files ?? {}).sort(), ['parts/logical.arc', 'parts/more.arc', 'requirements.arc']);
  assert.deepEqual(asked.sort(), ['parts/gone.arc', 'parts/logical.arc', 'parts/more.arc', 'requirements.arc']);
  assert.equal(await importClosure('main.arc', 'model M {}\n', read), null);
});

test('a model that reaches too many files stops at the limit', async () => {
  const read = async (path: string) => `import "${Number(path.replace(/\D/g, '')) + 1}.arc"\n`;
  const files = await importClosure('main.arc', 'import "1.arc"\n', read, { files: 5, characters: 1000 });
  assert.equal(Object.keys(files ?? {}).length, 5);
});
