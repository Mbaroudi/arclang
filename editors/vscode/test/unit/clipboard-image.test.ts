import assert from 'node:assert/strict';
import { test } from 'node:test';
import { clipboardCommands, writeImageToClipboard } from '../../src/workbench/clipboard-image';

test('each platform has its own tool, given the file as an argument and never through a shell', () => {
  const mac = clipboardCommands('darwin', '/tmp/x/diagram.png');
  assert.equal(mac[0].file, '/usr/bin/osascript');
  assert.match(mac[0].args[1], /«class PNGf»/);
  assert.match(mac[0].args[1], /POSIX file "\/tmp\/x\/diagram\.png"/);
  const windows = clipboardCommands('win32', 'C:\\t\\diagram.png');
  assert.equal(windows[0].file, 'powershell.exe');
  assert.equal(windows[0].args.at(-1), 'C:\\t\\diagram.png');
  assert.ok(!windows[0].args.slice(0, -1).some((a) => a.includes('diagram.png')), 'the path is not in the script');
  const linux = clipboardCommands('linux', '/tmp/d.png');
  assert.deepEqual(linux.map((c) => c.file), ['wl-copy', 'xclip']);
  assert.ok(linux.every((c) => c.stdin));
});

test('what is not a PNG never reaches the clipboard', async () => {
  await assert.rejects(writeImageToClipboard(Buffer.from('<svg/>'), 'darwin'), /not a PNG/);
});
