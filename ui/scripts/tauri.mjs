// `npm run tauri <cmd>` wrapper: runs the bundled @tauri-apps/cli with the Tauri
// project pinned to ../src-tauri, so it works from ui/ on any OS (tauri-cli only
// searches the current dir and its subfolders for tauri.conf.json).
import { run } from '@tauri-apps/cli';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
process.env.TAURI_APP_PATH ??= resolve(here, '../../src-tauri');
process.env.TAURI_FRONTEND_PATH ??= resolve(here, '..');

run(process.argv.slice(2), 'npm run tauri').catch((e) => {
  console.error(e);
  process.exit(1);
});
