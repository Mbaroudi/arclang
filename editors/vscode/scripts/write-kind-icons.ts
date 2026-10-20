// Writes the kind icons of src/pure/kind-icons.ts to media/kinds/<family>.svg,
// the files the side bar tree shows (a tree icon must be a file of the
// extension). `npm run icons` after a change; a unit test checks they agree.
import * as fs from 'node:fs';
import * as path from 'node:path';
import { iconFamilies, iconSvg } from '../src/pure/kind-icons';

const dir = path.resolve(__dirname, '..', 'media', 'kinds');
fs.mkdirSync(dir, { recursive: true });
for (const family of iconFamilies()) fs.writeFileSync(path.join(dir, `${family}.svg`), `${iconSvg(family)}\n`);
console.log(`${iconFamilies().length} icons written to ${path.relative(process.cwd(), dir)}`);
