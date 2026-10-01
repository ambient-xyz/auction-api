# Ambient Auction

This repo contains the datastructures that make up the Ambient auction program.
It is the shared wire-format crate used by the on-chain program, clients, and
services. Changes to instruction discriminators, instruction payloads, account
ordering, or account layouts are therefore protocol changes.

## Versioned bundle pricing

`AccountLayoutVersion::V6` adds per-job pricing to the bundle escrow flow. The
configured account-layout version accepts V6, while the production default
remains V5 so that deployment can be performed explicitly.

A V6 `BundleEscrowV2` account contains the complete V5 layout followed by a V6
tail. The tail stores a bitmap of pages whose pricing has been posted, whether
pricing has been sealed, and the pricing commitment supplied when the escrow was
opened.

A V6 `BundleVerifierPageV2` account likewise contains its complete V5 layout
followed by a V6 tail. Each pricing entry contains the job public key, the job's
maximum output-token count, and its price per output token. A page contains up to
six entries, and a bundle contains up to three pages or eighteen jobs.

`BundlePricingCommitmentV6Message` defines the canonical bytes committed by the
requester. It binds the bundle hash, ordered pricing entries, entry count, and the
zero-padded `ambient.bundle.pricing.v6` domain. Unused entries and reserved bytes
are zero-filled. Producers and consumers must preserve entry order and hash these
exact bytes.

The V6 flow introduces `OpenBundleEscrowV6` (30), `PostBundlePricingV6` (31), and
`SealBundlePricingV6` (32). Pricing is posted one verifier page at a time and
then sealed after all expected pages have been supplied and checked against the
commitment. `CommitAuctionSettlementV3` (33) commits the auction hash and winner,
but does not carry the single bundle-wide clearing price used by V2 settlement.

Verifier-page evidence hashing continues to cover only the original V1 page
prefix for V5 and V6 accounts. V5 rent metadata and V6 pricing metadata are not
part of those evidence bytes. V6 pricing is authenticated separately by the
pricing commitment.

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
