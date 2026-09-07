# Ambient Auction V2 interface

`ambient_auction_v2.json` is the Shank/Anchor-compatible source IDL. The
corresponding Codama tree is committed at `codama/ambient_auction_v2.json`.
Both describe the existing production V2 ABI; neither changes the program.

## Encoding

- Instruction data starts with one `u8` discriminator (`12` through `23`),
  followed by the listed argument fields in order.
- Integers use little-endian encoding. Public keys are 32 raw bytes.
- Reserved and padding fields are part of the ABI and must be zero unless a
  future interface version explicitly assigns them.
- The account lists in the IDL are the fixed prefixes. The tails below are
  context-dependent and must be appended explicitly by clients.
- `InitConfigPolicyV2` remains encodable, but the production program rejects it.
  It is available only in program artifacts built with
  `dev-config-policy-init`; production initializes the policy at genesis.

## Context-dependent accounts

All indexes below start after the fixed account prefix emitted by the IDL.

### `FinalizeBundleVerificationV2`

Normal finalization appends every canonical verifier page in ascending page
index. Each page is writable. A page-backed settlement may be authorized by
quorum signatures or by the configured service bypass.

Disputed finalization appends:

1. writable `BundleVerificationDisputeV2` PDA;
2. writable bond refund recipient stored in the dispute;
3. writable Solana incinerator (`1nc1nerator11111111111111111111111111111111`);
4. for every page index, a read-only dispute staging page followed by the
   writable canonical page.

The staging/canonical pairs are ordered by page index. A signed disputed
finalization may exceed the legacy transaction-size ceiling; callers must check
the complete transaction before submission.

### `ClaimVerifierLstakeV2`

Append all canonical verifier pages in ascending page index. They are writable
because a successful claim marks the corresponding reward entries claimed.

### `ExpireBundleEscrowV2`

A non-disputed expiry has no tail. A replacement-quorum timeout appends the
writable dispute PDA and writable bond refund recipient, then the writable
winner node only when restoring the original verified result requires its
payout.

### Optional fixed accounts

- `PostBundleResultV2` account 3 is the writable canonical page being populated.
  The older no-page encoding remains representable but is rejected by the
  production V2 settlement flow.
- `SelectBundleVerifiersV2` account 1 is absent for initial selection and is the
  writable dispute PDA for replacement selection.
- `InitBundleVerifierPageV2` uses the same layout for canonical and dispute
  staging pages. The selected PDA seed family determines which page is created.

## PDA seeds

All PDAs use program ID `Auction111111111111111111111111111111111111`.

| PDA | Seeds, in order |
| --- | --- |
| Config policy | UTF-8 `global_config`, UTF-8 `policy_v2` |
| Bundle escrow | UTF-8 `bundle_escrow_v2`, payer public key, 32-byte bundle hash, `bundle_version` as little-endian `u32` |
| Canonical verifier page | UTF-8 `bundle_verifier_page_v2`, bundle escrow public key, `page_index` as little-endian `u16` |
| Dispute staging page | UTF-8 `bundle_dispute_verifier_page_v2`, bundle escrow public key, `page_index` as little-endian `u16` |
| Verification dispute | UTF-8 `bundle_verification_dispute_v2`, bundle escrow public key |

The Codama IDL exposes these as native PDA nodes. The Shank IDL retains the same
information in `metadata.pdas` because Shank 0.4.8 does not emit seed metadata
in its legacy Anchor JSON schema.

## Account layouts

| Account | Byte layout | Size |
| --- | --- | ---: |
| `BundleEscrowV2` | 8-byte header, 496-byte POD payload, 64-byte typed reserved region | 568 |
| `BundleVerifierPageV2` | 8-byte header, 808-byte POD payload, 64 reserved bytes | 880 |
| `BundleVerificationDisputeV2` | 8-byte header, 184-byte POD payload | 192 |
| `ConfigPolicyV2` | direct POD policy layout (no account header) | 1568 |

The versioned account header is `discriminator: u8`, `version: u8`, and six
reserved zero bytes. The discriminators are `8` for escrow, `9` for page, and
`10` for dispute. The current production escrow/page layout is V2; the dispute
layout is V1.

## Regeneration

Install Rust, Node.js 20.18 or newer, and pnpm 11.19.0, then run:

```sh
pnpm install --frozen-lockfile
pnpm idl
```

`pnpm idl:check` regenerates into a temporary directory and fails if either
committed IDL differs byte-for-byte.
