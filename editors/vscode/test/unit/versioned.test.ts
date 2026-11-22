import assert from 'node:assert/strict';
import { test } from 'node:test';
import { chosenTarget, PerVersion, staleEdit } from '../../src/pure/versioned';

/** A promise and the hand that settles it. */
function held<T>(): { promise: Promise<T>; give: (value: T) => void; fail: (error: Error) => void } {
  let give!: (value: T) => void;
  let fail!: (error: Error) => void;
  const promise = new Promise<T>((resolve, reject) => {
    give = resolve;
    fail = reject;
  });
  return { promise, give, fail };
}

test('an edit computed on the text as it is now is applied; on an older text it is refused, and says why', () => {
  assert.equal(staleEdit(7, 7, 'pump.arc'), null);
  const refusal = staleEdit(7, 8, 'pump.arc');
  assert.ok(refusal && refusal.startsWith('pump.arc changed while the edit was being computed'), String(refusal));
  assert.match(String(refusal), /nothing was written/);
});

test('the file an export is written to is the one the extension chose, whatever the page sends back', () => {
  assert.equal(chosenTarget(undefined, '/etc/cron.d/owned'), undefined);
  assert.equal(chosenTarget('/work/out.pdf', '/etc/cron.d/owned'), '/work/out.pdf');
});

test('two asks for one version share one computation', async () => {
  const store = new PerVersion<string>();
  const first = held<string>();
  let runs = 0;
  const a = store.get('doc', 1, () => (runs++, first.promise));
  const b = store.get('doc', 1, () => (runs++, first.promise));
  assert.equal(a.started, true);
  assert.equal(b.started, false);
  first.give('v1');
  assert.equal(await a.work, 'v1');
  assert.equal(await b.work, 'v1');
  assert.equal(runs, 1);
  assert.equal(store.cached('doc', 1), 'v1');
});

test('an ask for a newer version does not get the computation under way for the older one', async () => {
  const store = new PerVersion<string>();
  const old = held<string>();
  const fresh = held<string>();
  const a = store.get('doc', 1, () => old.promise);
  const b = store.get('doc', 2, () => fresh.promise);
  assert.equal(b.started, true);
  fresh.give('v2');
  assert.equal(await b.work, 'v2');
  // The older one ends last: it is given to who asked, and does not replace the newer.
  old.give('v1');
  assert.equal(await a.work, 'v1');
  assert.equal(store.cached('doc', 2), 'v2');
  assert.equal(store.cached('doc', 1), undefined);
  assert.equal(store.last('doc'), 'v2');
  assert.equal(store.busy('doc'), false);
});

test('the older computation ending does not hide that the newer one is still under way', async () => {
  const store = new PerVersion<string>();
  const old = held<string>();
  const fresh = held<string>();
  const a = store.get('doc', 1, () => old.promise);
  store.get('doc', 2, () => fresh.promise);
  old.give('v1');
  await a.work;
  await Promise.resolve();
  assert.equal(store.busy('doc'), true);
  assert.equal(store.get('doc', 2, () => Promise.resolve('again')).started, false);
  fresh.give('v2');
});

test('a value that may not be kept is given and not kept; a computation that throws is kept by nobody', async () => {
  const store = new PerVersion<string>();
  assert.equal(await store.get('doc', 1, async () => 'given', () => false).work, 'given');
  assert.equal(store.cached('doc', 1), undefined);
  const broken = store.get('doc', 1, async () => {
    throw new Error('no compiler');
  });
  await assert.rejects(broken.work, /no compiler/);
  await Promise.resolve();
  assert.equal(store.busy('doc'), false);
  assert.equal(store.cached('doc', 1), undefined);
  assert.equal(await store.get('doc', 1, async () => 'now').work, 'now');
  assert.equal(store.cached('doc', 1), 'now');
});
