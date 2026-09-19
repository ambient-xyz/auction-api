# Ambient Auction

This repo contains the datastructures that make up the Ambient auction program.

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

## V2 IDLs

The production V2 interface is published as a
[Shank/Anchor-compatible IDL](idl/ambient_auction_v2.json) and a
[portable Codama IDL](codama/ambient_auction_v2.json). See the
[interface guide](idl/V2_INTERFACE.md) for dynamic page/dispute account tails
and PDA seeds.

Regenerate both artifacts with `pnpm idl` and verify them with
`pnpm idl:check`.
