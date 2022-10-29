#!/usr/bin/env bash
set -e

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}


steps=50
repeat=20
chain=kico
pallets=(
	sudo
  create-dao
  agency
  square
  square
)

for p in ${pallets[@]}
do
	cargo run --release --features runtime-benchmarks --bin tico benchmark pallet \
		--execution=wasm \
		--chain local \
		--wasm-execution=compiled \
		--pallet=daos_$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--template=./.maintain/pallet-weight-template.hbs \
		--output ./pallets/daos/$p/src/weights.rs
done