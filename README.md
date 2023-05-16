# pvf-checker

The purpose of this tool is to check before releasing a new polkadot binary all existing PVFs on Polkadot and Kusama to make sure the prechecking is passing in case we introduce some changes, e.g. by upgrading wasmtime.

See https://github.com/paritytech/polkadot/issues/7048 for more details.

### How it works

This tool uses [`subxt`](https://github.com/paritytech/subxt) to connect to an RPC node specified by the `--rpc-url` flag and query relevant runtime storage items from the relay chain including PVF for each parachain. Once it collected all PVFs, it will spawn a PVF worker and run the check for each PVF.

### Versioning

This tool will be published on crates.io following the same versioning scheme as polkadot.

Suggested usage (hypothetical):

```bash
for version in '0.9.41 0.9.42 0.9.43-rc1'; do
    cargo install pvf-checker --version $version
    pvf-checker pvf-check --rpc-url 'wss://kusama-rpc.polkadot.io:443' --skip 2268
done
```
