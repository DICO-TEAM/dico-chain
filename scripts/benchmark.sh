#!/usr/bin/env bash
set -e

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}

steps=50
repeat=20

chains=(
	tico
	kico
)

pallets=(
	frame_system
	pallet_balances
	pallet_timestamp
	pallet_membership
)


for chain in ${chains[@]}
do
    for pallet in ${pallets[@]}
    do
		cargo run --release --features runtime-benchmarks -- benchmark \
			--chain=$chain \
			--execution=wasm \
			--wasm-execution=compiled \
			--pallet=$pallet \
			--extrinsic='*' \
			--steps=$steps \
			--repeat=$repeat \
			--raw \
			--output=./runtime/$chain/src/weights/$pallet.rs
    done
done
