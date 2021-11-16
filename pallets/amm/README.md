# Automated Market Makers (AMM)

## Interface

### Dispatchable Functions

#### For general users

* `add_liquidity` - Add liquidity to previously created asset pair pool.
* `remove_liquidity` - Remove liquidity from specific liquidity pool in the form of burning shares.
* `swap_exact_assets_for_assets` - Use a fixed amount of supply assets to exchange for target assets not less than `amount_out_min`.
* `swap_assets_for_exact_assets` - Use no more than `amount_in_max` supply assets to exchange for a fixed amount of target assets.