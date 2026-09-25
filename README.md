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

Configuration accounts are unconditional. Remove the `global-config` Cargo feature from downstream manifests and build commands.
`RequestJobAccountKeys` and `RequestJobAccounts` always include `config` after `system_program`.
Account layouts, instruction numbers, and the `global_config` address seed remain unchanged.
The current auction program still rejects legacy instruction numbers `0..=11`, including `InitConfig`, before account parsing.
This Rust build-interface change requires no on-chain account migration.
