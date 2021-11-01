use sp_std::vec::Vec;

pub trait UpdateOraclesStorgage<AccountId, CurrencyId> {
	fn initialize_members(members: Vec<AccountId>);
	fn insert_members(memmber: &[AccountId]);
	fn del_members(memmber: &[AccountId]);
	fn unlock_price(currency_id: CurrencyId);
}
