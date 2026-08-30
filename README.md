# Ambient Auction

This repo contains the datastructures that make up the Ambient auction program.

## Building

This crate is largely intended to be a library. However, there is a debugging utiltity
called `decode-account` that allows for decoding raw or base64/base58/hex encoded
accounts and displaying their most relevant information as text.

All builds require `AMBIENT_AUCTION_PROGRAM_ID` to be set to the auction program's
base58 pubkey. The crate decodes it with `five8_const` at compile time, and there is
no fallback default, so missing or malformed values fail during compilation.

To build the library:

```shell
AMBIENT_AUCTION_PROGRAM_ID=Auction111111111111111111111111111111111111 cargo build
```

The binary is `cfg`-ed out by default and requires the `decoder` feature to be enabled
in order to build.

To build the binary:

```shell
AMBIENT_AUCTION_PROGRAM_ID=Auction111111111111111111111111111111111111 \
  cargo build --release --bin decode-account --features decoder
```
