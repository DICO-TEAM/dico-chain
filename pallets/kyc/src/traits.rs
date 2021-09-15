pub trait KycHandler<AccountId, AreaCode> {
    fn get_uesr_area(user: &AccountId) -> Option<AreaCode>;
}