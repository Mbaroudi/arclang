// The AI chat as an agent: the change written in the file is marked line by
// line (Keep / Undo), attached files are read (Word, ReqIF, XML) and sent as
// data, and the page may only send what the host expects.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { changeMarks } from '../../src/ai/core/diff';
import { attachmentsBlock, CREATE_MODEL, MAX_ATTACHED, NO_FILE_RULES, SYSTEM_RULES, WRITE_FILE, writablePath } from '../../src/ai/core/prompts';
import { parseAssistAnswer } from '../../src/ai/core/answer';
import { isQuestion, validateRequests, withoutMetaTalk } from '../../src/ai/core/requests';
import { converse } from '../../src/ai/core/conversation';
import { parseFromChatPage } from '../../src/ai/chat/protocol';
import { markupText, unzip, decode } from '../../webview/chat/markup';
import { inlineSteps } from '../../webview/chat/steps';

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

test('another system than the open model is a new model, never mixed into it', () => {
  const answer = parseAssistAnswer('', [{ id: 't', name: 'create_model', input: { explanation: 'A phone.', source: 'model Phone { }' } }]);
  assert.equal(answer.newModel, 'model Phone { }');
  assert.deepEqual(answer.requests, []);
  assert.match(SYSTEM_RULES, /never put it into the open model: call create_model/);
  assert.match(SYSTEM_RULES, /ask them in one sentence/);
  assert.deepEqual(CREATE_MODEL.parameters.required, ['explanation', 'source']);
});

test('asked to add something, the AI does it, files next to the model included', () => {
  assert.match(SYSTEM_RULES, /DO it now, in this answer, with the tools/);
  assert.match(SYSTEM_RULES, /Never answer "tell me if you want me to write it"/);
  assert.deepEqual(WRITE_FILE.parameters.required, ['path', 'content']);
  assert.equal(writablePath('physics/CalcPhysics.mo'), 'physics/CalcPhysics.mo');
  assert.equal(writablePath('./data/day.csv'), 'data/day.csv');
  for (const bad of ['../x.mo', '/etc/x.mo', 'physics/../../x.mo', '.git/config.json', 'model.arc', 'run.sh', '', 42]) assert.equal(writablePath(bad), null, String(bad));
});

test('a layer added "in" itself goes at the top level, without the layer it named', () => {
  const r = validateRequests([{ op: 'add', keyword: 'operational_analysis', name: 'Operational Analysis', layer: 'operational_analysis' }, { op: 'add', keyword: 'epbs', name: 'EPBS', container: 'OnePedalCar' }]);
  assert.ok(r.ok);
  if (!r.ok) return;
  for (const q of r.requests as { top_level?: boolean; layer?: string; container?: string }[]) {
    assert.equal(q.top_level, true);
    assert.equal(q.layer, undefined);
    assert.equal(q.container, undefined);
  }
});

test('an element added with no container and no layer goes at the top level', () => {
  const r = validateRequests([{ op: 'add', keyword: 'test_case', name: 'TC-1' }, { op: 'add', keyword: 'component', name: 'C', layer: 'logical_architecture' }, { op: 'add_raw', declaration: 'trace "A" satisfies "B" {}', top_level: 'true' }]);
  assert.ok(r.ok);
  if (!r.ok) return;
  assert.equal((r.requests[0] as { top_level?: boolean }).top_level, true);
  assert.equal((r.requests[1] as { top_level?: boolean }).top_level, undefined);
  assert.equal((r.requests[2] as { top_level?: boolean }).top_level, true);
});

test('"1. … 2. …" written in one line is read as a list; other numbers are not', () => {
  const steps = inlineSteps('Les étapes sont : 1. Donner une physique aux ports. 2. Ajouter les nœuds. 3. Lancer la simulation.');
  assert.deepEqual(steps, { lead: 'Les étapes sont :', items: ['Donner une physique aux ports.', 'Ajouter les nœuds.', 'Lancer la simulation.'] });
  assert.equal(inlineSteps('Une latence de 2. ms et 3. fois'), null);
  assert.equal(inlineSteps('Version 4.12.0 et 4.13.0'), null);
});

test('a file written in the same answer as the edits is written too', async () => {

  const provider = {
    async *chat() {
      yield { type: 'tool_call' as const, call: { id: 'a', name: 'write_file', input: { path: 'physics/P.mo', content: 'package P end P;' } } };
      yield { type: 'tool_call' as const, call: { id: 'b', name: 'propose_edits', input: { explanation: 'x', requests: [] } } };
    },
  };
  const ran: string[] = [];
  const result = await converse(provider as never, [{ role: 'user', content: 'add the simulation' }], {
    tools: [WRITE_FILE, { name: 'propose_edits', description: '', parameters: {} }],
    final: ['propose_edits'],
    run: async (call: { name: string }) => { ran.push(call.name); return 'ok'; },
  });
  assert.deepEqual(ran, ['write_file']);
  assert.deepEqual(result.calls.map((c: { name: string }) => c.name), ['propose_edits']);
});

test('an answer cut at the token limit is not run: the AI is asked again for a smaller part', async () => {
  const seen: string[] = [];
  let round = 0;
  const provider = {
    async *chat(messages: { role: string; content: string }[]) {
      round += 1;
      seen.push(messages[messages.length - 1].content);
      if (round === 1) {
        yield { type: 'tool_call' as const, call: { id: 'a', name: 'propose_edits', input: {} } };
        yield { type: 'done' as const, stopReason: 'max_tokens' };
        return;
      }
      yield { type: 'tool_call' as const, call: { id: 'b', name: 'propose_edits', input: { explanation: 'part 1. Next: the rest', requests: [] } } };
      yield { type: 'done' as const, stopReason: 'tool_use' };
    },
  };
  const result = await converse(provider as never, [{ role: 'user', content: 'everything' }], {
    tools: [{ name: 'propose_edits', description: '', parameters: {} }],
    final: ['propose_edits'],
    run: async () => 'ok',
  });
  assert.equal(round, 2);
  assert.match(seen[1], /cut at the token limit/);
  assert.equal((result.calls[0].input as { explanation: string }).explanation, 'part 1. Next: the rest');
});

test('a polite request is a request; a question is a question', () => {
  for (const asked of ['Can you simulate the battery range?', 'could you add the simulation', 'peux-tu ajouter la simulation ?', 'Pouvez-vous simuler la chauffe ?', 'tu peux simuler ça']) assert.equal(isQuestion(asked), false, asked);
  for (const asked of ['peut-on faire une co-simulation dans OMSimulator ?', 'How do I run it?', 'comment je lance la simulation', 'Can I bring my requirements from DOORS?', 'est ce que mon modèle est bon ?']) assert.equal(isQuestion(asked), true, asked);
  assert.equal(isQuestion('rajoute la simulation openmodelica'), false);
});

test('polite requests in French and English; the answer starts with the answer', () => {
  for (const asked of ['est-ce que tu peux ajouter un test ?', 'Would you add the tests?', 'vous pouvez simuler ça ?']) assert.equal(isQuestion(asked), false, asked);
  assert.equal(isQuestion('Which requirements are not yet verified by a test, and what would you add?'), true);
  assert.equal(withoutMetaTalk("Il s'agit d'une question : voici les étapes. Le gate indique…"), 'Le gate indique…');
  assert.equal(withoutMetaTalk("C'est une question : voici comment faire, sans rien modifier.\n\nLa Gate est en FAIL"), 'La Gate est en FAIL');
  assert.equal(withoutMetaTalk('This is a question, so here is the answer: run Simulate.'), 'run Simulate.');
  assert.equal(withoutMetaTalk('Le modèle compile.'), 'Le modèle compile.');
});

test('a request with a question mark is still a request', () => {
  for (const asked of ["fait moi une modelisation d'une voiture one-pedale ?", 'ajoute un capteur de pluie ?', 'Add a headlight?', 'stp corrige les erreurs ?', 'Model a drone?']) assert.equal(isQuestion(asked), false, asked);
  for (const asked of ['où sont les résultats ?', 'Where are the simulation results kept?', 'quels tests manquent ?']) assert.equal(isQuestion(asked), true, asked);
});
