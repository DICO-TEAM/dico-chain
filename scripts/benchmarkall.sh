#!/usr/bin/env bash
set -e

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}


steps=50
repeat=20
chain=kico
pallets=(
	amm
	currencies
	dao
	farm
	farm-extend
	ico
	kyc
	lbp
	nft
	oracle
	pricedao
	template
	treasury
)

for p in ${pallets[@]}
do
	cargo run --release --features runtime-benchmarks -- benchmark \
		--chain=$chain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=pallet_$p \
		--extrinsic='*' \
		--steps=$steps \
		--repeat=$repeat \
		--template=./.maintain/pallet-weight-template.hbs \
		--output ./pallets/$p/src/weights.rs
done