import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import {
  accountLinkNode,
  accountNode,
  accountValueNode,
  argumentValueNode,
  bytesTypeNode,
  constantDiscriminatorNode,
  constantNode,
  constantPdaSeedNodeFromString,
  constantValueNode,
  createFromRoot,
  fixedSizeTypeNode,
  numberTypeNode,
  numberValueNode,
  pdaLinkNode,
  pdaNode,
  pdaSeedValueNode,
  pdaValueNode,
  publicKeyTypeNode,
  publicKeyValueNode,
  stringTypeNode,
  stringValueNode,
  variablePdaSeedNode,
} from 'codama';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const args = process.argv.slice(2);
const check = args.includes('--check');
const outIndex = args.indexOf('--out-dir');
const outputRoot = outIndex === -1 ? root : resolve(args[outIndex + 1]);
const temporary = mkdtempSync(join(tmpdir(), 'ambient-auction-v2-idl-'));
const rawPath = join(temporary, 'shank.json');
const globalRawPath = join(temporary, 'shank-global-config.json');

const extract = (features, path) => {
  const generated = spawnSync(
    'cargo',
    [
      'run',
      '--quiet',
      '--locked',
      '--features',
      features,
      '--example',
      'generate_v2_idl',
      '--',
      path,
    ],
    { cwd: root, encoding: 'utf8', stdio: ['ignore', 'inherit', 'inherit'] },
  );
  if (generated.status !== 0) process.exit(generated.status ?? 1);
};
extract('idl', rawPath);
extract('idl,global-config', globalRawPath);
if (readFileSync(rawPath, 'utf8') !== readFileSync(globalRawPath, 'utf8')) {
  throw new Error('default and global-config builds expose different V2 interfaces');
}

const shank = JSON.parse(readFileSync(rawPath, 'utf8'));
shank.name = 'ambient_auction_v2';

const constants = [
  ['PROGRAM_ID', 'publicKey', 'Auction111111111111111111111111111111111111'],
  ['CONFIG_SEED', 'string', 'global_config'],
  ['CONFIG_POLICY_V2_SEED', 'string', 'policy_v2'],
  ['BUNDLE_ESCROW_V2_SEED', 'string', 'bundle_escrow_v2'],
  ['BUNDLE_VERIFIER_PAGE_V2_SEED', 'string', 'bundle_verifier_page_v2'],
  ['BUNDLE_DISPUTE_VERIFIER_PAGE_V2_SEED', 'string', 'bundle_dispute_verifier_page_v2'],
  ['BUNDLE_VERIFICATION_DISPUTE_V2_SEED', 'string', 'bundle_verification_dispute_v2'],
  ['MAX_VERIFIERS_PER_AUCTION', 'u64', 3],
  ['MAX_BUNDLE_VERIFIER_PAGE_V2_ENTRIES', 'u64', 6],
  ['BUNDLE_ESCROW_V2_ACCOUNT_SIZE', 'u64', 568],
  ['BUNDLE_VERIFIER_PAGE_V2_ACCOUNT_SIZE', 'u64', 880],
  ['BUNDLE_VERIFICATION_DISPUTE_V2_ACCOUNT_SIZE', 'u64', 192],
  ['CONFIG_POLICY_V2_ACCOUNT_SIZE', 'u64', 1568],
  ['ACCOUNT_LAYOUT_VERSION_V1', 'u8', 1],
  ['ACCOUNT_LAYOUT_VERSION_V2', 'u8', 2],
  ['VERIFICATION_VERDICT_V2_UNSET', 'u8', 0],
  ['VERIFICATION_VERDICT_V2_VERIFIED', 'u8', 1],
  ['VERIFICATION_VERDICT_V2_REJECTED', 'u8', 2],
  ['BUNDLE_VERIFICATION_DISPUTE_V2_KIND_UNSET', 'u8', 0],
  ['BUNDLE_VERIFICATION_DISPUTE_V2_KIND_MISSED_VERIFICATION', 'u8', 1],
  ['BUNDLE_VERIFICATION_DISPUTE_V2_KIND_PAID_VERDICT_DISPUTE', 'u8', 2],
  ['BUNDLE_ESCROW_V2_STATUS_OPEN', 'u64', 0],
  ['BUNDLE_ESCROW_V2_STATUS_AWARDED', 'u64', 1],
  ['BUNDLE_ESCROW_V2_STATUS_RESULT_POSTED', 'u64', 2],
  ['BUNDLE_ESCROW_V2_STATUS_FINALIZED_VERIFIED', 'u64', 3],
  ['BUNDLE_ESCROW_V2_STATUS_FINALIZED_REJECTED', 'u64', 4],
  ['BUNDLE_ESCROW_V2_STATUS_EXPIRED', 'u64', 5],
  ['BUNDLE_ESCROW_V2_STATUS_PROVISIONAL_VERIFIED', 'u64', 6],
  ['BUNDLE_ESCROW_V2_STATUS_PROVISIONAL_REJECTED', 'u64', 7],
  ['BUNDLE_ESCROW_V2_STATUS_DISPUTED', 'u64', 8],
];

shank.constants = constants.map(([name, type, value]) => ({
  name,
  type,
  value: type === 'string' || type === 'publicKey' ? JSON.stringify(value) : String(value),
}));
shank.metadata = {
  ...shank.metadata,
  dynamicAccounts: 'See idl/V2_INTERFACE.md',
  pdas: [
    ['configPolicyV2', ['global_config', 'policy_v2']],
    ['bundleEscrowV2', ['bundle_escrow_v2', 'payer', 'bundle_hash', 'bundle_version_le']],
    ['bundleVerifierPageV2', ['bundle_verifier_page_v2', 'bundle_escrow', 'page_index_le']],
    ['bundleDisputeVerifierPageV2', ['bundle_dispute_verifier_page_v2', 'bundle_escrow', 'page_index_le']],
    ['bundleVerificationDisputeV2', ['bundle_verification_dispute_v2', 'bundle_escrow']],
  ].map(([name, seeds]) => ({ name, seeds })),
};

const literal = (value) => constantPdaSeedNodeFromString('utf8', value);
const pdas = [
  pdaNode({ name: 'configPolicyV2', seeds: [literal('global_config'), literal('policy_v2')] }),
  pdaNode({
    name: 'bundleEscrowV2',
    seeds: [
      literal('bundle_escrow_v2'),
      variablePdaSeedNode('payer', publicKeyTypeNode()),
      variablePdaSeedNode('bundleHash', fixedSizeTypeNode(bytesTypeNode(), 32)),
      variablePdaSeedNode('bundleVersion', numberTypeNode('u32')),
    ],
  }),
  pdaNode({
    name: 'bundleVerifierPageV2',
    seeds: [
      literal('bundle_verifier_page_v2'),
      variablePdaSeedNode('bundleEscrow', publicKeyTypeNode()),
      variablePdaSeedNode('pageIndex', numberTypeNode('u16')),
    ],
  }),
  pdaNode({
    name: 'bundleDisputeVerifierPageV2',
    seeds: [
      literal('bundle_dispute_verifier_page_v2'),
      variablePdaSeedNode('bundleEscrow', publicKeyTypeNode()),
      variablePdaSeedNode('pageIndex', numberTypeNode('u16')),
    ],
  }),
  pdaNode({
    name: 'bundleVerificationDisputeV2',
    seeds: [
      literal('bundle_verification_dispute_v2'),
      variablePdaSeedNode('bundleEscrow', publicKeyTypeNode()),
    ],
  }),
];

const nativeConstant = ([name, type, value]) => {
  if (type === 'publicKey') return constantNode(name, publicKeyTypeNode(), publicKeyValueNode(value));
  if (type === 'string') return constantNode(name, stringTypeNode('utf8'), stringValueNode(value));
  return constantNode(name, numberTypeNode(type), numberValueNode(value));
};

const source = createFromRoot(rootNodeFromAnchor(shank)).getRoot();
const accountPdas = {
  bundleEscrowV2: 'bundleEscrowV2',
  bundleVerifierPageV2: 'bundleVerifierPageV2',
  bundleVerificationDisputeV2: 'bundleVerificationDisputeV2',
  configPolicyV2: 'configPolicyV2',
};
const accountDiscriminators = { bundleEscrowV2: 8, bundleVerifierPageV2: 9, bundleVerificationDisputeV2: 10 };
const accounts = source.program.accounts.map((account) => accountNode({
  ...account,
  pda: pdaLinkNode(accountPdas[account.name]),
  discriminators: accountDiscriminators[account.name] === undefined
    ? undefined
    : [constantDiscriminatorNode(constantValueNode(numberTypeNode('u8'), numberValueNode(accountDiscriminators[account.name])))],
}));

const pdaSeeds = (...entries) => entries.map(([name, value]) => pdaSeedValueNode(name, value));
const instructions = source.program.instructions.map((instruction) => ({
  ...instruction,
  docs: instruction.docs ?? (
    ['finalizeBundleVerificationV2', 'claimVerifierLstakeV2', 'expireBundleEscrowV2'].includes(instruction.name)
      ? ['This instruction has context-dependent trailing accounts documented in idl/V2_INTERFACE.md.']
      : undefined
  ),
  accounts: instruction.accounts.map((account) => {
    let defaultValue = account.defaultValue;
    if (account.name === 'configPolicy') defaultValue = pdaValueNode('configPolicyV2');
    if (instruction.name === 'openBundleEscrowV2' && account.name === 'bundleEscrow') {
      defaultValue = pdaValueNode('bundleEscrowV2', pdaSeeds(
        ['payer', accountValueNode('payer')],
        ['bundleHash', argumentValueNode('bundleHash')],
        ['bundleVersion', argumentValueNode('bundleVersion')],
      ));
    }
    if (instruction.name === 'initBundleVerifierPageV2' && account.name === 'bundleVerifierPage') {
      defaultValue = pdaValueNode('bundleVerifierPageV2', pdaSeeds(
        ['bundleEscrow', accountValueNode('bundleEscrow')],
        ['pageIndex', argumentValueNode('pageIndex')],
      ));
    }
    if (instruction.name === 'disputeBundleVerificationV2' && account.name === 'bundleVerificationDispute') {
      defaultValue = pdaValueNode('bundleVerificationDisputeV2', pdaSeeds(
        ['bundleEscrow', accountValueNode('bundleEscrow')],
      ));
    }
    const accountLink = accountPdas[account.name] ? accountLinkNode(accountPdas[account.name]) : account.accountLink;
    return { ...account, ...(defaultValue ? { defaultValue } : {}), ...(accountLink ? { accountLink } : {}) };
  }),
}));

const codama = {
  ...source,
  program: {
    ...source.program,
    name: 'ambientAuctionV2',
    docs: ['Portable production V2 interface. Dynamic account tails are documented in idl/V2_INTERFACE.md.'],
    accounts,
    instructions,
    pdas,
    constants: constants.map(nativeConstant),
  },
};

const outputs = new Map([
  [join(outputRoot, 'idl', 'ambient_auction_v2.json'), `${JSON.stringify(shank, null, 2)}\n`],
  [join(outputRoot, 'codama', 'ambient_auction_v2.json'), `${JSON.stringify(codama, null, 2)}\n`],
]);

for (const [path, contents] of outputs) {
  if (check) {
    if (readFileSync(path, 'utf8') !== contents) {
      console.error(`${path} is stale; run pnpm idl`);
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, contents);
  }
}

rmSync(temporary, { recursive: true, force: true });
