## Supported Runtimes
  - Polkadot
  - Kusama
  - Westend
  - Aleph Zero testnet
  - Aleph Zero mainnet

TODO: Improve the runtimes implementation without the need of replicating the same functions for each runtime. Note that *RuntimeApi* is runtime specific. It gives access to api functions specific for each runtime.

## Generated files from subxt-cli

Download metadata from a substrate node, for use with `subxt` codegen.

```bash
subxt metadata --url wss://westend-rpc.polkadot.io:443 -f bytes > westend_metadata.scale
subxt metadata --url wss://kusama-rpc.polkadot.io:443 -f bytes > kusama_metadata.scale
subxt metadata --url wss://rpc.polkadot.io:443 -f bytes > polkadot_metadata.scale
```

Generate runtime API client code from metadata.

```bash
subxt codegen --file polkadot_metadata.scale | rustfmt --edition=2018 --emit=stdout > polkadot_runtime.rs
subxt codegen --file kusama_metadata.scale | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
subxt codegen --file westend_metadata.scale | rustfmt --edition=2018 --emit=stdout > westend_runtime.rs
```
