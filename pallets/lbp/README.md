# Liquidity Bootstrapping Pool (LBP)

## Interface

### Dispatchable Functions

#### For general users
* `create_lbp` - Create a liquidity bootstrapping pool.
* `exit_lbp` - Close a liquidity bootstrapping pool.
* `swap_exact_amount_supply` - Use a fixed amount of supply assets to exchange for target assets not less than `min_target_amount`.
* `swap_exact_amount_target` - Use no more than `max_supply_amount` supply assets to exchange for a fixed amount of target assets.

#### For council users
* `add_fundraising_asset` - Add support for fundraising assets.
* `remove_fundraising_asset` - Remove support for fundraising assets.
