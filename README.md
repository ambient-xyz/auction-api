# Ambient Auction

This repo contains the datastructures that make up the Ambient auction program.
It is the shared wire-format crate used by the on-chain program, clients, and
services. Changes to instruction discriminators, instruction payloads, account
ordering, or account layouts are therefore protocol changes.

## Versioned bundle pricing

`AccountLayoutVersion::V4` combines lifecycle, Small credit, and pricing metadata.
Lifecycle metadata records page allocation, deadlines, and credit amounts.
V4 escrows use 656 bytes, and V4 verifier pages use 1,200 bytes.
Both readers reject headers 5 and 6 and retired lengths of 616 and 904 bytes.
Existing V1, V2, and SmallV3 layouts and instruction encodings stay unchanged.
The policy default selects V4. `OpenBundleEscrowV4` keeps instruction number 26.
`OpenPricedBundleEscrowV4` keeps instruction number 30.
Instruction numbers, payload bytes, and account order stay unchanged.

A pricing commitment is a hash of the agreed job prices.
Hash the exact bytes from `BundlePricingCommitmentV4Message::as_bytes()`.
The message contains the zero-padded `ambient.bundle.pricing.v6` domain, bundle
hash, entry count, and ordered pricing entries. Unused entries and reserved bytes
are zero-filled.

For V4 pages, evidence hashes cover the V1 prefix and all six input-token values.
They exclude rent and pricing metadata. The pricing commitment authenticates pricing separately.
The `lifecycle()` and `pricing()` accessors read separate metadata slices.
Dispute accounts expose `evidence()` and `evidence_mut()`.
The API removes former V5 and V6 source names without compatibility aliases.
The pricing domain retains its literal `ambient.bundle.pricing.v6` bytes to preserve commitment hashes.
Other signature domains stay unchanged. New evidence hashes include account header 4.

`ConfigPolicyV2` exposes `small_credit_mint`, `small_credit_enabled`,
`small_credit_slash_authority`, and `small_credit_slash_sequence` in Rust and JSON.
Their byte offsets remain 1320, 1352, 1384, and 1416.
The `_reserved_small_credit_enabled` and `_reserved_small_credit_slash_sequence`
fields preserve padding in the former 32-byte slots.
The three former `reserved_words` at offset 1448 now hold `replay: ReplayPolicyV1` (see below).
The layout byte remains at 1544, and the account remains 1,568 bytes.
Existing policy bytes and helper signatures stay unchanged.
New default policies change only the layout byte from 5 to 4.
Consumers must adopt these V4 source names before deployment.

## Exact replay (phase 6)

These types support exact-replay verification. Existing accounts and instruction encodings stay unchanged.

- **`ConfigPolicyV2.replay: ReplayPolicyV1`** (96 bytes at offset 1448, formerly three zero reserved words).
  - Contents: the replay class id, the exact-replay tiers (`tier_mask`, bit `1 << tier`, never Small), the hold in
    slots, the off-chain arbitration size, and penalty terms per `ReplayFaultKind` (a forfeit mask and suspension
    hours).
  - Version 0 means off, with every byte zero. Every deployed policy is therefore off until an admin sets it.
  - Admins set it with `SetConfigPolicyV2` patch kind 8 (`REPLAY_SETTINGS`), carried only by the 104-byte
    `SetConfigPolicyReplayV1Args`.
- **`BundleEscrowLifecycleV4Data.exact_replay`** (escrow byte 570, formerly reserved). Set at open for a replay
  tier. Such an escrow finalizes only with verifier signatures, never through the service bypass. Its provisional
  window is at least the hold.
- **`ReplayAnchorV1`** (discriminator 11, 480 bytes; PDA `[b"replay_anchor_v1", miner node, SHA-256(engine id)]`)
  and **`AnchorReplaySpansV1`** (instruction 34).
  - A miner's node key anchors the SHA-256 of an engine's latest span commit. Span ids must increase strictly.
  - Commits chain by `prev_commit_sha256`, so the anchor fixes every earlier span.
  - The account keeps the last eight anchors with their slots.
- **Errors** 81 `InvalidReplayPolicy`, 82 `InvalidReplayAnchor` and 83 `ReplayAnchorOutOfOrder`. Code 80 is left for the
  in-program verifier selection error (`InvalidVerifierSelectionAccounts`).

## Closing a settled escrow (ENG-352)

- **`CloseBundleEscrowV4`** (instruction 39, empty payload; accounts: `payer` signer and writable, `bundle_escrow`
  writable) returns a V4 escrow's rent to the account that paid for it.
  - The payer is the account the escrow PDA `[b"bundle_escrow_v2", payer, bundle_hash, bundle_version]` was derived
    from. It must sign.
  - The escrow must be terminal (`FinalizedVerified`, `FinalizedRejected` or `Expired`), past its claim deadline,
    and have no verifier pages left (`allocated_page_bitmap == 0`; close them with `CloseBundleVerifierPageV4` first).
  - It must hold no lamports beyond its rent-exempt minimum (error 6 `NonZeroBalance`), so unsettled funds are never
    paid to the payer.
- Number 39 is free on every open branch: `main` uses 34 and 35 for the Small escrow instructions, ENG-255 uses 34
  and ENG-256 uses 35 to 38. No new account layout or error code.

## Building

This crate is largely intended to be a library. However, there is a debugging utiltity
called `decode-account` that allows for decoding raw or base64/base58/hex encoded
accounts and displaying their most relevant information as text.

The binary is `cfg`-ed out by default and requires the `decoder` feature to be enabled
in order to build.

To build the binary:

```shell
cargo build --release --bin decode-account --features decoder
```

Configuration accounts are unconditional. Remove the `global-config` Cargo feature from downstream manifests and build commands.
`RequestJobAccountKeys` and `RequestJobAccounts` always include `config` after `system_program`.
That build-interface change did not renumber existing instructions, alter existing account layouts, or change the `global_config` address seed.
The current auction program still rejects legacy instruction numbers `0..=11`, including `InitConfig`, before account parsing.
This Rust build-interface change requires no on-chain account migration.
