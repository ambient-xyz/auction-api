# Add or change an instruction

Every instruction that the auction program accepts needs a parser and tests in `ambient-auction-interface`.
The `Interface compatibility` workflow tests the proposed API code against interface `main`.
It also makes sure that Cargo uses this API checkout rather than the pinned dependency.

1. Update the API instruction definition and its encoding tests.
2. Add the matching interface parser and account validation tests.
3. Pin the proposed API commit in the interface pull request.
4. Merge the interface pull request after `Interface tests` passes.
5. Rerun `Interface compatibility` on this API pull request.
6. Merge this API pull request after that check passes.

The interface test match must name each API instruction explicitly.
A new variant without an interface case fails compilation.
For new argument formats under an existing instruction number, add cases for each accepted format.

The `Transaction parser` workflow builds the optional `parser` feature on every API pull request.
Its instruction match names every variant. A new variant without a parser case fails compilation.
When instruction fields or accepted formats change, update the parser and its expected JSON tests in the same pull request.
Keep cases for historical formats that RPC clients still read.
