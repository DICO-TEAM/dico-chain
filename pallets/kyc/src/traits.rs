pub trait KycHandler<AccountId, AreaCode> {
	fn get_user_area(user: &AccountId) -> Option<AreaCode>;
}
