// The properties pane in a browser, through the real compiler: typed
// controls, the page's own checks and the compiler's before anything is
// written, one edit per field, inherited values redefined and reset,
// relationships and traces as links, findings, the Markdown preview,
// several elements at once, a tree element dropped on the diagram.
// SHOTS=dir keeps pictures of the pane in light and dark.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const BASE = `http://127.0.0.1:${process.env.PORT || 8766}`;
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(58)} ${detail || ''}`); };

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  const errors = []; page.on('pageerror', (e) => errors.push(String(e)));
  await page.goto(`${BASE}/e2e.html`); await page.waitForSelector('.arcviz-stage svg', { timeout: 30000 }); await page.waitForTimeout(600);
  const source = () => page.evaluate(() => fetch('/source').then((r) => r.text()));
  const reset = async () => { await page.evaluate(() => fetch('/reset')); await page.evaluate(() => window.__push()); await page.waitForTimeout(1000); };
  const count = () => page.evaluate(() => window.__results.length);
  const last = () => page.evaluate(() => window.__results.at(-1));
  const posted = (type) => page.evaluate((t) => window.__posted.filter((m) => m.type === t).length, type);
  const waitEdit = async (before) => { for (let i = 0; i < 50 && (await count()) === before; i++) await page.waitForTimeout(150); await page.waitForTimeout(900); return (await count()) === before ? null : last(); };
  const select = async (id) => { await page.evaluate((x) => window.postMessage({ type: 'select', id: x }, '*'), id); await page.waitForTimeout(700); };
  const row = (key) => page.locator(`.wb-properties tr[data-key="${key}"]`).first();
  const message = (key) => row(key).locator('.wb-field-msg');
  const unfold = async (section) => { const s = page.locator(`.wb-sec[data-section="${section}"]`); if (await s.evaluate((n) => n.classList.contains('is-folded')).catch(() => false)) await s.locator('.wb-sec-head').click(); };

  // 1. The sections of a function, with the icon of its kind
  await reset(); await page.locator('.wb-layer', { hasText: 'System' }).first().click(); await page.waitForTimeout(800); await select('SF-DET');
  const sections = await page.$$eval('.wb-properties .wb-sec', (s) => s.map((n) => n.dataset.section));
  report('sections: identity, groups, inherited, relationships, traces', ['identity', 'timing', 'inherited', 'relationships', 'traces', 'description'].every((s) => sections.includes(s)), sections.join(','));
  report('the kind has its icon in the header', (await page.locator('.wb-props-title svg.wb-kind-function').count()) === 1);
  report('every field explains itself (metamodel documentation)', (await row('latency').locator('.wb-tip').textContent()).includes('Quantity (Time)'));
  const located = await page.locator('.wb-source-line').textContent();
  report('the source line is shown with Go to source', /line \d+/.test(located) && /Go to source/.test(located), located);

  // 2. A quantity of the wrong dimension: refused before anything is written
  let n = await count();
  const latency = row('latency').locator('input.wb-quantity-value');
  await latency.fill('5 kg'); await page.waitForTimeout(200);
  const wrong = await message('latency').textContent();
  report('a mass typed for a time is refused as it is typed', /kg measures mass/.test(wrong) && (await row('latency').evaluate((r) => r.classList.contains('is-invalid'))), wrong);
  await latency.press('Enter'); await page.waitForTimeout(600);
  report('… and Enter writes nothing', (await count()) === n);
  // 3. The right value: one edit, applied on Enter
  await latency.fill('35'); await page.waitForTimeout(500);
  await latency.press('Enter'); let r = await waitEdit(n); let text = await source();
  report('a quantity applies on Enter, with the unit of the picker', !!r?.ok && /"SF-DET"[\s\S]*?latency: 35 ms/.test(text) && r.requests.length === 1, r?.ok ? '' : r?.error);
  // 4. The unit picker
  n = await count(); await row('wcet').locator('select.wb-unit').selectOption('us'); r = await waitEdit(n); text = await source();
  report('changing the unit writes the quantity in that unit', !!r?.ok && /wcet: 25 us/.test(text), r?.ok ? '' : r?.error);

  // 5. An enumeration
  await unfold('safety'); n = await count(); await row('safety_level').locator('select').selectOption('ASIL-C'); r = await waitEdit(n); text = await source();
  report('an enumeration is a list of its values, quoted when it must be', !!r?.ok && /"SF-DET"[\s\S]*?safety_level: "ASIL-C"/.test(text), r?.ok ? '' : r?.error);

  // 6. A reference: the picker searches the right kind, a wrong kind is refused live
  const is = row('is').locator('input.wb-picker-input');
  await is.fill('Driver'); await page.waitForTimeout(500);
  const kindMsg = await message('is').textContent();
  report('a reference to an element of another kind is refused live', /is an actor; Is points at a type/.test(kindMsg), kindMsg);
  await is.fill(''); await is.focus(); await page.waitForTimeout(200);
  const options = await page.$$eval('.wb-picker-list:not([hidden]) .wb-picker-option', (o) => o.map((x) => x.textContent));
  report('the reference picker lists the elements of the kind it points at', options.length >= 1 && options.every((o) => /Periodic task/.test(o)), options.join(' | '));
  await is.press('Escape');
  report('the reference has a jump link to its element', (await row('is').locator('.wb-jump').isEnabled()));
  await row('is').locator('.wb-jump').click(); await page.waitForTimeout(700);
  report('… which selects the type', (await page.locator('.wb-name-input').inputValue()) === 'Periodic task');

  // 7. A requirement's parent through the picker (keyboard)
  await select('REQ-LKA-002'); await unfold('links');
  const parent = row('parent').locator('input.wb-picker-input');
  await parent.fill('LKA-001'); await page.waitForTimeout(300); n = await count();
  await parent.press('ArrowDown'); await parent.press('Enter'); r = await waitEdit(n); text = await source();
  report('a reference picked with the keyboard is written', !!r?.ok && /"REQ-LKA-002"[\s\S]*?parent: "REQ-LKA-001"/.test(text), r?.ok ? '' : r?.error);

  // 8. The compiler's own refusal, live, before apply (a trace kind the grammar refuses)
  await select('SF-DET'); await unfold('traces');
  await page.locator('.wb-trace-add select').selectOption('verifies');
  const traceInput = page.locator('.wb-trace-add input'); await traceInput.fill('REQ-LKA-001'); await page.waitForTimeout(1800);
  const live = await page.locator('.wb-trace-add .wb-field-msg').textContent();
  report('the compiler\'s refusal shows on the field before anything is applied', /refused|Expected trace/.test(live) && (await posted('panes-validate')) > 0, live.slice(0, 90));
  await traceInput.fill(''); await page.keyboard.press('Escape');

  // 9. Inherited from the type: greyed, Redefine, Reset to inherited
  await select('SF-DET'); await unfold('inherited');
  const inherited = row('period');
  report('a value of the type shows greyed with where it comes from', (await inherited.evaluate((r) => r.classList.contains('is-inherited'))) && /20 ms/.test(await inherited.textContent()) && /Periodic task/.test(await inherited.textContent()));
  await inherited.locator('button', { hasText: 'Redefine' }).click(); await page.waitForTimeout(300);
  const own = row('period').locator('input.wb-quantity-value'); await own.fill('10'); n = await count(); await own.press('Enter'); r = await waitEdit(n); text = await source();
  report('Redefine writes the element\'s own value', !!r?.ok && /"SF-DET"[\s\S]*?period: 10 ms/.test(text), r?.ok ? '' : r?.error);
  await unfold('inherited');
  report('a redefined value says what it redefines', /redefines Periodic task: 20 ms/.test(await row('period').textContent()));
  n = await count(); await row('period').locator('button[aria-label="Reset to inherited"]').click(); r = await waitEdit(n); text = await source();
  const block = /function "Detect lane markings" \{[^}]*\}/.exec(text)?.[0] ?? '';
  report('Reset to inherited takes the element\'s own value away', !!r?.ok && !/period/.test(block) && (await row('period').evaluate((x) => x.classList.contains('is-inherited'))), r?.ok ? '' : r?.error);

  // 10. A refusal of the compiler on apply stays on the field, with what was typed
  await select('LC-CAM'); await unfold('structure'); n = await count();
  await row('variant').locator('.wb-switch').click(); r = await waitEdit(n); await page.waitForTimeout(400);
  const kept = await message('variant').textContent().catch(() => '');
  report('a refusal on apply is kept on its field', r && !r.ok && /variation/.test(kept), kept.slice(0, 80));

  // 11. Relationships and traces, as links
  await unfold('relationships');
  const rels = await page.locator('.wb-sec[data-section="relationships"]').textContent();
  report('relationships list owned, incoming and outgoing', /Owned \(\d\)/.test(rels) && /Capture frame/.test(rels) && /Component exchange LaneFlow/.test(rels), rels.slice(0, 120));
  await unfold('traces'); const traces = await page.locator('.wb-sec[data-section="traces"]').textContent();
  report('traces list what it satisfies', /Satisfies \(1\)/.test(traces) && /REQ-LKA-001/.test(traces), traces.slice(0, 120));
  await select('REQ-LKA-001'); await unfold('traces');
  const verified = await page.locator('.wb-sec[data-section="traces"]').textContent();
  report('a requirement shows what verifies it and satisfies it', /Verified by/.test(verified) && /TC-LKA-001/.test(verified) && /Satisfied by/.test(verified), verified.slice(0, 140));
  await page.locator('.wb-sec[data-section="traces"] .wb-elem-link', { hasText: 'TC-LKA-001' }).first().click(); await page.waitForTimeout(600);
  report('a click on a trace selects the other end', (await page.locator('.wb-name-input').inputValue()) === 'TC-LKA-001');

  // 12. The Markdown preview of the description
  await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(900);
  await select('LC-CTL'); const area = page.locator('.wb-desc-block[data-key="description"] textarea');
  if (await area.isHidden()) await page.locator('.wb-desc-block[data-key="description"] .wb-preview-toggle').click();
  n = await count(); await area.fill('Plans the **steering** correction:\n- within `80 ms`\n- releases on override'); await area.evaluate((e) => e.blur()); r = await waitEdit(n); await page.waitForTimeout(600);
  const preview = page.locator('.wb-desc-block[data-key="description"] .wb-markdown');
  report('the description applies on leaving it and shows as Markdown', !!r?.ok && (await preview.locator('strong').textContent()) === 'steering' && (await preview.locator('li').count()) === 2, r?.ok ? '' : r?.error);

  // 13. Findings on the element, with the fix the compiler names
  await reset(); await select('REQ-LKA-001');
  n = await count(); await page.evaluate(() => fetch('/edit', { method: 'POST', body: JSON.stringify({ requests: [{ op: 'remove', element: 'TC-LKA-001' }, { op: 'remove_trace', from: 'LC-CTL', kind: 'satisfies', to: 'REQ-LKA-001' }, { op: 'remove_trace', from: 'LC-CAM', kind: 'satisfies', to: 'REQ-LKA-001' }] }) }));
  await page.evaluate(() => window.__push()); await page.waitForTimeout(1500); await select('REQ-LKA-001');
  const findings = await page.locator('.wb-sec[data-section="findings"]').textContent().catch(() => '');
  report('the findings on the element have their own section', /trace\.requirement/.test(findings), findings.slice(0, 100));
  report('… with the fix the compiler names', (await page.locator('.wb-sec[data-section="findings"] .wb-fix').count()) > 0);

  // 14. Several elements: the common attributes, one value on all
  await reset(); await page.locator('.wb-layer', { hasText: 'Logical' }).first().click(); await page.waitForTimeout(900);
  const click = async (id, shift) => { const b = await page.locator(`.arcviz-stage [data-id="${id}"]`).first().boundingBox(); if (shift) await page.keyboard.down('Shift'); await page.mouse.click(b.x + 30, b.y + 8); if (shift) await page.keyboard.up('Shift'); await page.waitForTimeout(400); };
  await click('LC-CAM'); await click('LC-ACT', true); await page.waitForTimeout(400);
  report('a multi-selection shows what the elements have in common', /2 elements selected/.test(await page.locator('.wb-properties').textContent()) && (await page.locator('.wb-bulk tr[data-key="safety_level"]').count()) === 1);
  n = await count(); await page.locator('.wb-bulk tr[data-key="safety_level"] select').selectOption('ASIL-D'); r = await waitEdit(n); text = await source();
  report('… and sets one value on all of them in one edit', !!r?.ok && r.requests.length === 2 && (text.match(/safety_level: "ASIL-D"/g) || []).length === 2, r?.ok ? '' : r?.error);

  // 15. A tree element dropped on a declared diagram is added to what it shows
  await reset(); await page.evaluate(() => window.postMessage({ type: 'show', view: 'lab:Lane keeping chain', select: null }, '*')); await page.waitForTimeout(1200);
  n = await count();
  await page.evaluate(() => { const dt = new DataTransfer(); dt.setData('application/vnd.code.tree.arclangmodel', JSON.stringify({ id: 'PN-CAM' })); const sheet = document.querySelector('.wb-sheet'); sheet.dispatchEvent(new DragEvent('dragover', { dataTransfer: dt, bubbles: true, cancelable: true })); sheet.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true })); });
  r = await waitEdit(n); text = await source();
  report('a tree element dropped on a declared diagram joins its shows', !!r?.ok && /diagram "Lane keeping chain" \{[^}]*shows: \[[^\]]*"PN-CAM"/s.test(text), r?.ok ? '' : r?.error);
  n = await count();
  await page.evaluate(() => window.postMessage({ type: 'panes-drop', id: 'LC-CTL' }, '*')); await page.waitForTimeout(800);
  report('one already drawn there is revealed, nothing written', (await count()) === n && (await page.locator('.wb-name-input').inputValue()) === 'LKA Controller');

  // 16. Keyboard: the section heads move with the arrows and fold with Enter
  await select('LC-CTL'); await page.locator('.wb-sec-head').first().focus(); await page.keyboard.press('ArrowDown');
  const second = await page.evaluate(() => document.activeElement?.closest('.wb-sec')?.dataset.section);
  await page.keyboard.press('Enter'); const folded = await page.locator(`.wb-sec[data-section="${second}"]`).evaluate((s) => s.classList.contains('is-folded'));
  report('section heads: arrows move between them, Enter folds', second && second !== 'identity' && folded, second);
  await page.keyboard.press('Enter');

  if (process.env.SHOTS) {
    await select('SF-DET'); for (const s of ['inherited', 'traces']) await unfold(s);
    const pane = await page.locator('.wb-right').boundingBox();
    await page.screenshot({ path: `${process.env.SHOTS}/pane-light.png`, clip: pane });
    await page.evaluate(() => document.documentElement.classList.add('dark'));
  }

  report('no page error', errors.length === 0, errors.join(' | ').slice(0, 200));
  const failed = rows.filter((x) => !x.ok).length;
  console.log(`\n${rows.length - failed}/${rows.length} panes checks passed`);
  await browser.close();
  process.exit(failed ? 1 : 0);
})();
