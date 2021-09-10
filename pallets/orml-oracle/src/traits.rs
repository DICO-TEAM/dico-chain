use sp_std::vec::Vec;

pub trait UpdateFeedMembers<AccountId> {
    fn initialize_members(members: Vec<AccountId>);
    fn insert_members(memmber: &[AccountId]);
    fn del_members(memmber: &[AccountId]);
}
