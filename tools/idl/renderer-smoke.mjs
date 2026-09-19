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
const jsOut = join(temporary, 'js-out');
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

// Type-check generated JS.
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

// Compile generated JS to CommonJS so Node can resolve directory imports.
writeFileSync(join(jsRoot, 'package.json'), '{"type": "commonjs"}');
mkdirSync(jsOut, { recursive: true });
writeFileSync(join(jsOut, 'package.json'), '{"type": "commonjs"}');
run('pnpm', [
  'exec',
  'tsc',
  '--outDir', jsOut,
  '--target', 'ES2022',
  '--module', 'CommonJS',
  '--moduleResolution', 'Node',
  '--types', 'node',
  '--skipLibCheck',
  '--esModuleInterop',
  join(jsRoot, 'src', 'generated', 'index.ts'),
]);

// JavaScript behavioral checks: verify account counts and parsing for
// optional-account instructions using the "omitted" strategy.
const jsSmoke = `
const { getSelectBundleVerifiersV2Instruction, parseSelectBundleVerifiersV2Instruction } = require('${join(jsOut, 'instructions', 'selectBundleVerifiersV2.js').replace(/\\/g, '/')}');
const { getPostBundleResultV2Instruction } = require('${join(jsOut, 'instructions', 'postBundleResultV2.js').replace(/\\/g, '/')}');

const DUMMY_ESCROW = '11111111111111111111111111111112';
const DUMMY_DISPUTE = '11111111111111111111111111111113';
const DUMMY_AUTHORITY = '11111111111111111111111111111114';
const DUMMY_PAGE = '11111111111111111111111111111115';

// SelectBundleVerifiersV2: initial selection emits one account.
const initialIx = getSelectBundleVerifiersV2Instruction({ bundleEscrow: DUMMY_ESCROW });
if (initialIx.accounts.length !== 1) {
  console.error('FAIL: initial selection should have 1 account, got', initialIx.accounts.length);
  process.exit(1);
}

// SelectBundleVerifiersV2: replacement selection emits two accounts.
const replacementIx = getSelectBundleVerifiersV2Instruction({
  bundleEscrow: DUMMY_ESCROW,
  bundleVerificationDispute: DUMMY_DISPUTE,
});
if (replacementIx.accounts.length !== 2) {
  console.error('FAIL: replacement selection should have 2 accounts, got', replacementIx.accounts.length);
  process.exit(1);
}
if (replacementIx.accounts[1].address !== DUMMY_DISPUTE) {
  console.error('FAIL: replacement account 1 should be the dispute PDA');
  process.exit(1);
}

// JS parses both forms.
const parsedInitial = parseSelectBundleVerifiersV2Instruction(initialIx);
if (parsedInitial.accounts.bundleVerificationDispute !== undefined) {
  console.error('FAIL: parsed initial selection should have no dispute account');
  process.exit(1);
}
const parsedReplacement = parseSelectBundleVerifiersV2Instruction(replacementIx);
if (parsedReplacement.accounts.bundleVerificationDispute?.address !== DUMMY_DISPUTE) {
  console.error('FAIL: parsed replacement selection should have dispute account');
  process.exit(1);
}

// PostBundleResultV2: without page emits 3 accounts.
const DUMMY_PUBKEY = '11111111111111111111111111111111';
const emptyEntries = Array.from({ length: 6 }, () => ({
  jobId: DUMMY_PUBKEY,
  postedOutputTokens: 0n,
  acceptedOutputTokens: 0n,
  assignedVerifiersTokenRanges: [0n, 0n, 0n, 0n, 0n, 0n],
  verifierRewardTokens: [0n, 0n, 0n],
  verdict: 0,
  verifierClaimedBitmap: 0,
  reserved: new Uint8Array(6),
}));
const noPageIx = getPostBundleResultV2Instruction({
  authority: { address: DUMMY_AUTHORITY },
  bundleEscrow: DUMMY_ESCROW,
  configPolicy: DUMMY_ESCROW,
  resultHash: new Uint8Array(32),
  postedOutputTokens: 0n,
  pageIndex: 0,
  pageEntryCount: 0,
  reserved: new Uint8Array(4),
  pageEntries: emptyEntries,
});
if (noPageIx.accounts.length !== 3) {
  console.error('FAIL: post result without page should have 3 accounts, got', noPageIx.accounts.length);
  process.exit(1);
}

// PostBundleResultV2: with page emits 4 accounts.
const withPageIx = getPostBundleResultV2Instruction({
  authority: { address: DUMMY_AUTHORITY },
  bundleEscrow: DUMMY_ESCROW,
  configPolicy: DUMMY_ESCROW,
  bundleVerifierPage: DUMMY_PAGE,
  resultHash: new Uint8Array(32),
  postedOutputTokens: 0n,
  pageIndex: 0,
  pageEntryCount: 0,
  reserved: new Uint8Array(4),
  pageEntries: emptyEntries,
});
if (withPageIx.accounts.length !== 4) {
  console.error('FAIL: post result with page should have 4 accounts, got', withPageIx.accounts.length);
  process.exit(1);
}
if (withPageIx.accounts[3].address !== DUMMY_PAGE) {
  console.error('FAIL: post result account 3 should be the page');
  process.exit(1);
}

console.log('JS builder behavioral checks passed');
`;
writeFileSync(join(temporary, 'js-smoke.cjs'), jsSmoke);
run('node', [join(temporary, 'js-smoke.cjs')]);

// Rust behavioral checks: verify account counts for the same instructions.
const rustSmoke = `
use ambient_auction_v2_idl_smoke::instructions::*;
use ambient_auction_v2_idl_smoke::types::BundleVerifierPageV2Entry;
use solana_address::Address;

fn empty_entry() -> BundleVerifierPageV2Entry {
    BundleVerifierPageV2Entry {
        job_id: Address::default(),
        posted_output_tokens: 0,
        accepted_output_tokens: 0,
        assigned_verifiers_token_ranges: [0; 6],
        verifier_reward_tokens: [0; 3],
        verdict: 0,
        verifier_claimed_bitmap: 0,
        reserved: [0; 6],
    }
}

#[test]
fn select_initial_emits_one_account() {
    let ix = SelectBundleVerifiersV2Builder::new()
        .bundle_escrow(Address::default())
        .instruction();
    assert_eq!(ix.accounts.len(), 1);
}

#[test]
fn select_replacement_emits_two_accounts() {
    let ix = SelectBundleVerifiersV2Builder::new()
        .bundle_escrow(Address::default())
        .bundle_verification_dispute(Some(Address::default()))
        .instruction();
    assert_eq!(ix.accounts.len(), 2);
}

#[test]
fn post_result_without_page_emits_three_accounts() {
    let ix = PostBundleResultV2 {
        authority: Address::default(),
        bundle_escrow: Address::default(),
        config_policy: Address::default(),
        bundle_verifier_page: None,
    };
    let args = PostBundleResultV2InstructionArgs {
        result_hash: [0u8; 32],
        posted_output_tokens: 0,
        page_index: 0,
        page_entry_count: 0,
        reserved: [0u8; 4],
        page_entries: std::array::from_fn(|_| empty_entry()),
    };
    let ix = ix.instruction(args);
    assert_eq!(ix.accounts.len(), 3);
}

#[test]
fn post_result_with_page_emits_four_accounts() {
    let ix = PostBundleResultV2 {
        authority: Address::default(),
        bundle_escrow: Address::default(),
        config_policy: Address::default(),
        bundle_verifier_page: Some(Address::default()),
    };
    let args = PostBundleResultV2InstructionArgs {
        result_hash: [0u8; 32],
        posted_output_tokens: 0,
        page_index: 0,
        page_entry_count: 0,
        reserved: [0u8; 4],
        page_entries: std::array::from_fn(|_| empty_entry()),
    };
    let ix = ix.instruction(args);
    assert_eq!(ix.accounts.len(), 4);
}
`;
mkdirSync(join(rustRoot, 'tests'), { recursive: true });
writeFileSync(join(rustRoot, 'tests', 'smoke.rs'), rustSmoke);
writeFileSync(
  join(rustRoot, 'Cargo.toml'),
  readFileSync(join(rustRoot, 'Cargo.toml'), 'utf8') + '\n[[test]]\nname = "smoke"\npath = "tests/smoke.rs"\n',
);

run('cargo', ['test', '--quiet', '--manifest-path', join(rustRoot, 'Cargo.toml'), '--test', 'smoke'], {
  env: { ...process.env, CARGO_TARGET_DIR: join(temporary, 'cargo-target') },
});

rmSync(temporary, { recursive: true, force: true });
