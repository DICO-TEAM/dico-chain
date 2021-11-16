# Farm

## Interface

### Dispatchable Functions

#### For general users
* `deposit_lp` - Deposit liquid assets to designated mining pools to participate in mining.
* `withdraw_lp` - Withdraw liquidity.

#### For council users
* `set_halving_period` - Set the mining reward halving cycle,the unit is the number of blocks.
* `set_dico_per_block` - Set the reward for each block when starting mining.
* `set_start_block` - Set the block number of the mining pool to start mining.
* `update_pool_alloc_point` - Update the allocated points of each designated mining pool.
* `create_pool` - Create a new mining pool.