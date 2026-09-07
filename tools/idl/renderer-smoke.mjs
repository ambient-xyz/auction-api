import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createFromJson } from 'codama';
import { renderVisitor as renderJavaScript } from '@codama/renderers-js';
import { renderVisitor as renderRust } from '@codama/renderers-rust';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
mkdirSync(join(root, 'target'), { recursive: true });
const temporary = mkdtempSync(join(root, 'target', 'idl-render-'));
const jsRoot = join(temporary, 'js');
const rustRoot = join(temporary, 'rust');
mkdirSync(jsRoot, { recursive: true });
mkdirSync(join(rustRoot, 'src'), { recursive: true });
writeFileSync(
  join(rustRoot, 'Cargo.toml'),
  '[package]\nname = "ambient-auction-v2-idl-smoke"\nversion = "0.0.0"\nedition = "2021"\n\n[features]\nfetch = []\n\n[dependencies]\nnum-traits = "^0.2"\n',
);
writeFileSync(join(rustRoot, 'src', 'lib.rs'), 'pub mod generated;\npub use generated::*;\n');

const codama = createFromJson(readFileSync(join(root, 'codama', 'ambient_auction_v2.json'), 'utf8'));
await codama.accept(renderJavaScript(jsRoot, { formatCode: false, syncPackageJson: false }));
codama.accept(renderRust(rustRoot, {
  anchorTraits: false,
  formatCode: false,
  syncCargoToml: true,
}));

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    stdio: 'inherit',
    ...options,
  });
  if (result.status !== 0) {
    console.error(`renderer smoke artifacts retained at ${temporary}`);
    process.exit(result.status ?? 1);
  }
};

run('pnpm', [
  'exec',
  'tsc',
  '--noEmit',
  '--target',
  'ES2022',
  '--module',
  'ESNext',
  '--moduleResolution',
  'Bundler',
  '--types',
  'node',
  '--skipLibCheck',
  join(jsRoot, 'src', 'generated', 'index.ts'),
]);
run('cargo', ['check', '--quiet', '--manifest-path', join(rustRoot, 'Cargo.toml')], {
  env: { ...process.env, CARGO_TARGET_DIR: join(temporary, 'cargo-target') },
});
rmSync(temporary, { recursive: true, force: true });
