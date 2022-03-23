# Changelog
All notable changes to this project will be documented in this file.

## [unreleased]

### Update

- Add KusdCurrencyId

## [1.2.1] - 2022-02-18

### Bug Fixes

- Fix the bug that nft module can not compile and update runtime/lib.rs

### Performance

- Upgrade to polkadot-v0.9.16

### Build

- Add srtool.sh for build runtime

## [1.2.0] - 2022-01-31

### Bug Fixes

- Add root into funder set of lbp and farm pallets.


### Features

- Modify the TODO configuration
- Add benchmark script
- Generate weight with becnhmark
- Set kico parachain id

### Update

- Update the weight of nft module for testing


## [1.1.4] - 2022-01-04

### Bug Fixes

- Fix rpc expose error
- Support MultiNativeAsset for IsReserve
- Update vesting logic
- Add NeedRest logic

### Features

- Add TICO for release testnet

## [1.1.3] - 2021-12-23

### Bug Fixes

- Fix(farm-extend): Update `acc_reward_per_share` in the same block
- Upgrade to polkadot-v0.9.13
- Fix test bug

### Features

- Complete cross chain

### Upgrade

- Upgrade the currencies and the treasury module to latest version


## [1.1.2] - 2021-12-09

### Bug Fixes

- Fix area statistics error (#133)
- Clear_kyc_should_work
- Adjust steps.
- Add test case
- Update delele ias/SwordHolder logic
- Check whether the total deposit amount is zero.

### Features

- Add GenesisConfig for the currencies module

### Performance

- Fix period for develop (#140)
- Add vesting account for develop (#141)
- Release v1.1.2

### Testing

- Add the readme and test

### Update

- Add PendingInfo and VestingScheduleOf

## [1.1.1] - 2021-11-22

### Bug Fixes

- Fix command format

### Features

- Kico add vecting

### Performance

- Finish testnetwork (#14)

## [1.1.0] - 2021-11-18

### Bug Fixes

- Get price from swap or oracle
- Fix kyc sword holder information error
- Fix merge main conflict
- Remove reserved asset for module when add liquidity.
- Modify price type
- Support the sale of assets.
- Modify type definition.
- Modify the symbol of liquid assets.
- Pay the handling fee when user participate in crowdfunding.
- Modify PoolInfo structure field.
- Assets price div decimal
- Forbidden to add liquid assets to the liquidity pool.
- Forbidden to add liquid assets to the liquidity pool.
- Solve u32 mul decimal overflow bug
- Fix lbp pair create bug.
- Modify `LbpPair` struct.
- Update pool's alloc point in `on_finalize` function.
- Solve currency id is 0,get overflow bug
- Update pool's alloc point in `on_finalize` function.
- Delete duplicate references.
- Feed_price event add moment
- Fix the error of updating the pool when the alloc point is set to 0
- Fix weights
- Fix the bug
- #54
- Fix ias_do_provide_judgement judgement status
- Fix kychander
- Add InProcess Error
- Solving the halving problem
- Solving the halving problem
- Complete issue# 116
- Clear KYC info when delete ias and SwordHolder (#117)
- Build amm pallet benchmarking.(#122)
- Add pallets for kico network.
- Add farm pallet rpc.
- Fix rpc build error

### Documentation

- Add:How to Contribute
- Add Development wiki
- Add PULL_REQUEST_TEMPLATE.md (#119)
- Add README.md file.

### Feat

- Can not create nft that it has no permission
- Add AllTokensHash
- Remove the logo url in the IcoInfo

### Features

- Add the rpc method 'can_join_amount' in the ico module
- Fix the bug
- Resolve the conflict
- Update json
- Done todo
- Resolve the conflict
- Fix the bug
- Add the rpc interface for querying the user's earnings to be withdrawn.
- Change usdt currency id into 5
- Optimize the code
- Add basic functionality to NFT module development
- Add InviteInfoOf in ico module
- Add the rpc method get_token_price
- Add runtime_print in the func 'get_token_price'
- Update json
- Add the nft module
- Complete the NFT module
- `PoolInfo` struct add `start_block` and `end_block` field.
- Add the func 'Active' in the nft module
- Add license
- Add `PoolStatus` enum.
- Update alloc point.
- Update
- Add fundraising asset config
- Update
- Add runtime_print for testing
- Update the func get_token_price
- Return tuple type velue in the rpc method 'can_join_amount'
- Add `create_pool` extrinsic and unit test.
- Add `deposit_asset` extrinsic and unit test.
- Add `withdraw_asset` extrinsic and unit test.
- Integrate the pallet-farm-extend into the runtime
- Add benchmarking and weights
- Add IAS/Sword holder logout  logic
- Add the currencies module to the kico network
- Update to kico
- Add the ico module to the kico network

### Fix

- Fix the rpc bugs

### Miscellaneous Tasks

- Fix get price bug

### Performance

- Change chain_spec.rs logic
- Rm chglog
- Add git-cliff-action
- When the amount is equal to 0, there is no need to call the transfer interface.
- Fix format

### Refactor

- Use variables instead of constants.
- Add network node type primitives
- Update rust version (#114)
- Add vecting (#123)

### Testing

- Fix amm tests.
- Add `swap_exact_amount_target` unit tests.
- Fix tests error

### Update

- Update the README.md of the ico module

### Build

- Build farm pallet benchmarking.(#122)
- Remove useless code.

### Ci

- Add DepositBalanceInfo json type

### Revert

- Revert a previous commit

### Runtime

- Fix rpc builder

### Update

- Update the README.md
- Update the mock
- Update the test
- Use substrate3.0 syntax
- Update the cargo.toml

## [1.0.1] - 2021-09-14

### Bug Fixes

- Add area data
- To obtain asset information from the currencies pallet when add liquidity.

### Features

- Add the dao, ico and dico-treasury module
- Add the RPC and API in ico module
- Add oralce and price pallet

### Ci

- Add jenkins file
- Update makefile

## [1.0.0] - 2021-09-13

### Bug Fixes

- Fix init version of pallet

### Documentation

- Update readme

### Features

- Completed the basic functions of kyc
- Complete kyc pallet

### Ci

- Issue/ci template

<!-- generated by git-cliff -->
