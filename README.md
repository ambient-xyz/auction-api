# Ambient Auction

This repo contains the datastructures that make up the Ambient auction program.
It is the shared wire-format crate used by the on-chain program, clients, and
services. Changes to instruction discriminators, instruction payloads, account
ordering, or account layouts are therefore protocol changes.

## Versioned bundle pricing

`AccountLayoutVersion::V5` combines lifecycle, Small credit, and pricing metadata.
V5 escrows use 656 bytes, and V5 verifier pages use 1,200 bytes.
Both readers reject header 6 and the former V5 lengths of 616 and 904 bytes.
Version number 6 remains reserved without an account format.
Existing V1, V2, and SmallV3 layouts and instruction encodings stay unchanged.
The policy default remains V5. Pricing instructions 30 through 33 remain disabled
in the program until their handlers are complete. Instruction versions
change when their inputs, accounts, or meaning change. An account layout change
alone does not require another instruction.

A pricing commitment is a hash of the agreed job prices.
Hash the exact bytes from `BundlePricingCommitmentV6Message::as_bytes()`.
The message contains the zero-padded `ambient.bundle.pricing.v6` domain, bundle
hash, entry count, and ordered pricing entries. Unused entries and reserved bytes
are zero-filled.

For V5 pages, evidence hashes cover the V1 prefix and all six input-token values.
They exclude rent and pricing metadata. The pricing commitment authenticates pricing separately.
The `lifecycle()` and `pricing()` accessors read separate metadata slices.
The existing `v5()` and `v6()` getters remain compatibility wrappers.
The pricing types and signature domain keep their existing V6 names and bytes.

`ConfigPolicyV2` exposes `small_credit_mint`, `small_credit_enabled`,
`small_credit_slash_authority`, and `small_credit_slash_sequence` in Rust and JSON.
Their byte offsets remain 1320, 1352, 1384, and 1416.
The `_reserved_small_credit_enabled` and `_reserved_small_credit_slash_sequence`
fields preserve padding in the former 32-byte slots.
Three unused `reserved_words` remain at offset 1448.
The layout byte remains at 1544, and the account remains 1,568 bytes.
Existing policy bytes and helper signatures stay unchanged.

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
