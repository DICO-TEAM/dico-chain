# Development

## setup

If you'd like to build from source, first install Rust.

```
curl https://sh.rustup.rs -sSf | sh
```
or

```
make setup
```

finished. install build supports package(Ubuntu, optional for polkadot build)

```
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```
## Init Submodule
```angular2html
make submodule
```
## build

Once the development environment is set up.

```
make build
```

## Run local Node for development

```asm
make dev
```

