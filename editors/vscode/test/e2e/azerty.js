// Keys as a finger on a French (Mac AZERTY) keyboard sends them: the key
// labeled Z sends the code US keyboards call KeyW, and so on. Playwright's
// keyboard.press('Meta+z') sends KeyZ, which VS Code, reading the real
// layout of this Mac, takes for ⌘W. These go through the Chrome DevTools
// protocol with the code and the key of the French layout.
const LETTERS = { a: 'KeyQ', q: 'KeyA', z: 'KeyW', w: 'KeyZ', m: 'Semicolon' };
// '-' is on the key US calls Equal, '=' (and '+' with Shift) on Slash; the digits need Shift.
const SPECIAL = {
  '0': { key: 'à', shifted: '0', code: 'Digit0', keyCode: 48 },
  '1': { key: '&', shifted: '1', code: 'Digit1', keyCode: 49 },
  '=': { key: '=', code: 'Slash', keyCode: 187 },
  '+': { key: '+', code: 'Slash', keyCode: 187, shift: true },
  '-': { key: '-', code: 'Equal', keyCode: 189 },
};
const NAMED = { Escape: 27, Delete: 46, Backspace: 8, Enter: 13, Tab: 9, F2: 113, ArrowDown: 40, ArrowUp: 38, ArrowLeft: 37, ArrowRight: 39, End: 35, Home: 36, Space: 32 };
const MODIFIERS = { Alt: [1, 'Alt', 'AltLeft', 18], Control: [2, 'Control', 'ControlLeft', 17], Meta: [4, 'Meta', 'MetaLeft', 91], Shift: [8, 'Shift', 'ShiftLeft', 16] };

/** A `press('Meta+Shift+z')` for the window `win`, AZERTY-faithful. */
async function azerty(win) {
  const cdp = await win.context().newCDPSession(win);
  const send = (event) => cdp.send('Input.dispatchKeyEvent', event);
  return async (combo) => {
    const parts = combo.split('+');
    let last = parts.pop();
    if (last === '' && combo.endsWith('+')) { parts.pop(); last = '+'; }
    let mods = 0;
    for (const name of parts) {
      const [bit, key, code, keyCode] = MODIFIERS[name];
      mods |= bit;
      await send({ type: 'rawKeyDown', modifiers: mods, key, code, windowsVirtualKeyCode: keyCode });
    }
    let spec;
    if (SPECIAL[last]) {
      spec = { ...SPECIAL[last] };
      if (spec.shift) mods |= 8;
      if (mods & 8 && spec.shifted) spec.key = spec.shifted;
    } else if (/^[a-z]$/i.test(last)) {
      const c = last.toLowerCase();
      spec = { key: mods & 8 ? c.toUpperCase() : c, code: LETTERS[c] || `Key${c.toUpperCase()}`, keyCode: c.toUpperCase().charCodeAt(0) };
    } else {
      const key = last === 'Space' ? ' ' : last;
      spec = { key, code: last === 'Space' ? 'Space' : last, keyCode: NAMED[last] };
    }
    const text = !(mods & 6) && spec.key.length === 1 ? spec.key : undefined;
    await send({ type: text ? 'keyDown' : 'rawKeyDown', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode, text, unmodifiedText: text });
    await send({ type: 'keyUp', modifiers: mods, key: spec.key, code: spec.code, windowsVirtualKeyCode: spec.keyCode });
    for (const name of parts.reverse()) {
      const [bit, key, code, keyCode] = MODIFIERS[name];
      mods &= ~bit;
      await send({ type: 'keyUp', modifiers: mods, key, code, windowsVirtualKeyCode: keyCode });
    }
  };
}

module.exports = { azerty };
