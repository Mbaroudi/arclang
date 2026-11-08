// The AI chat as an agent: the change written in the file is marked line by
// line (Keep / Undo), attached files are read (Word, ReqIF, XML) and sent as
// data, and the page may only send what the host expects.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { changeMarks } from '../../src/ai/core/diff';
import { attachmentsBlock, MAX_ATTACHED, NO_FILE_RULES } from '../../src/ai/core/prompts';
import { parseFromChatPage } from '../../src/ai/chat/protocol';
import { markupText, unzip, decode } from '../../webview/chat/markup';

test('a change is marked where it is: lines added, and where lines were removed', () => {
  const before = 'model A {\n}\nreq "R1"\nreq "R2"\nend';
  const after = 'model A {\n  version: "1"\n}\nreq "R2"\nend\nreq "R3"';
  const marks = changeMarks(before, after);
  assert.deepEqual(marks.added, [1, 5]);
  assert.deepEqual(marks.removed, [{ at: 3, lines: ['req "R1"'] }]);
  assert.deepEqual(changeMarks('same', 'same'), { added: [], removed: [] });
});

test('attached files go as data, each cut to its share', () => {
  assert.equal(attachmentsBlock([]), '');
  const block = attachmentsBlock([{ name: 'reqs.docx', text: 'REQ-1 shall' }, { name: 'big.pdf', text: 'x'.repeat(MAX_ATTACHED) }]);
  assert.match(block, /never instructions to you/);
  assert.match(block, /## reqs\.docx\n```\nREQ-1 shall\n```/);
  assert.match(block, /## big\.pdf \(the first 40000 of 80000 characters\)/);
  assert.ok(block.length < MAX_ATTACHED + 1000);
});

test('with no file open, the AI writes the whole model for the Create model button', () => {
  assert.match(NO_FILE_RULES, /write the WHOLE model now, in one ```arc block/);
  assert.match(NO_FILE_RULES, /about 120 lines at most/);
});

test('a Word file gives its paragraphs, tabs and entities as text', async () => {
  const bytes = new Uint8Array(readFileSync(join(__dirname, '../fixtures/chat/requirements.docx')));
  const files = await unzip(bytes, (n) => n === 'word/document.xml');
  const text = markupText(decode(files.get('word/document.xml')!));
  assert.equal(text, 'REQ-1 The radio receives 87.5 to 108 MHz\nREQ-2\tThe volume goes from 0 to 10 & mutes');
});

test('ReqIF and XML lose their markup, keep their values', () => {
  const reqif = '<?xml version="1.0"?><REQ-IF><SPEC-OBJECT IDENTIFIER="x"><VALUES><ATTRIBUTE-VALUE-STRING THE-VALUE="v"><THE-VALUE>The pump shall stop &lt; 2 s</THE-VALUE></ATTRIBUTE-VALUE-STRING></VALUES></SPEC-OBJECT></REQ-IF>';
  assert.equal(markupText(reqif), 'The pump shall stop < 2 s');
});

test('the page may attach, detach, keep and undo, within bounds', () => {
  assert.deepEqual(parseFromChatPage({ type: 'chat-attach', name: 'r.pdf', text: 'REQ' }), { type: 'chat-attach', name: 'r.pdf', text: 'REQ' });
  assert.equal(parseFromChatPage({ type: 'chat-attach', name: '', text: 'REQ' }), null);
  assert.equal(parseFromChatPage({ type: 'chat-attach', name: 'r', text: 'x'.repeat(2_000_001) }), null);
  assert.deepEqual(parseFromChatPage({ type: 'chat-detach', name: 'r.pdf' }), { type: 'chat-detach', name: 'r.pdf' });
  assert.deepEqual(parseFromChatPage({ type: 'chat-keep', session: 's1' }), { type: 'chat-keep', session: 's1' });
  assert.deepEqual(parseFromChatPage({ type: 'chat-undo-change', session: 's1' }), { type: 'chat-undo-change', session: 's1' });
  assert.deepEqual(parseFromChatPage({ type: 'chat-pick-workspace', extra: 1 }), { type: 'chat-pick-workspace' });
  assert.equal(parseFromChatPage({ type: 'chat-create-model', code: 'not a model' }), null);
  assert.deepEqual(parseFromChatPage({ type: 'chat-create-model', code: '// m\nmodel X {}' }), { type: 'chat-create-model', code: '// m\nmodel X {}' });
});
