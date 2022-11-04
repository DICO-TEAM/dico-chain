<p align="center">
  <img src="docs/assets/dico-logo-small.png?raw=true" alt="image"/>
</p>

<h3 align="center">Decentralized and governable ICO platform</h3>

<div align="center">


[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![GitHub license](https://img.shields.io/badge/license-MIT%2FApache2-blue)](LICENSE)

</div>


## Credits

[ORML](https://github.com/open-web3-stack/open-runtime-module-library)

[paritytech/substrate](https://github.com/paritytech/substrate)

## Development
### setup
```asm
curl https://sh.rustup.rs -sSf | sh
```
```asm
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```
### clone projects
```asm
git clone https://github.com/paritytech/polkadot.git 
git clone https://github.com/DICO-TEAM/dico-chain.git
cd dico-chain
make submodule
```
### build 
Execute the following command in dico-chain and polkadot files.
```asm
cargo build --release
```
### run testnet
Execute the following command in the dico-chain file
```asm
cd launch
polkadot-launch config.json
```
### 
## Contributions

Contributors are welcomed to join this project. Please check [CONTRIBUTING](./.github/CONTRIBUTING.md) about how to contribute
to this project.

## Connect to mainnet

```angular2html
./kico --collator  -- --execution wasm --chain kusama
```

## License

The project is made available under the [Apache2.0](./LICENSE-APACHE)/[MIT](./LICENSE-MIT) license.
