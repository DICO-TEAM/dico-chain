use codec::{Encode, Decode};
use sp_runtime::{RuntimeDebug, Percent};
use node_primitives::{USDT, Balance};
use sp_std::{prelude::*,};

/// 额外信息
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Additional<AssetId, BlockNumber, BTreeSet>{
	/// ico对应的资产id
	pub asset_id: AssetId,
	/// ico结束时间
	pub end_time: BlockNumber,
	/// 已经募集到的usdt数量
	pub already_raise_usdt: USDT,
	/// 参与募集资金的人员名单
	pub people: BTreeSet,
}


/// 地址
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Address<AccountId>{
	/// usdt地址
	pub usdt: Option<Vec<u8>>,
	/// dico地址
	pub dico: Option<AccountId>,
}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum AddressEnum<AccountId> {
	Usdt(Vec<u8>),
	Dico(AccountId),
}

impl<T> Default for AddressEnum<T> {
	fn default() -> Self {
		Self::Usdt(vec![])
	}
}


#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct TokenAmount<AddressEnum> {
	pub usdt: (AddressEnum, Balance, USDT),
	pub dico: (AddressEnum, Balance, USDT),
}


/// 项目募集资金的具体金额（ 包括项目方和用户)
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct RaiseAmount<TokenAmount, BTreeMap> {
	pub project_manager_get: TokenAmount,
	pub others_send: BTreeMap,
}


/// 币种
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum Symbol {
	Usdt,
	Dico,
}

/// 投票
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DicoVote {
	Approval,
	Reject,
}

/// 募集资金的信息
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct IcoInfo<Balance, BlockNumber, Address>{
	/// 项目名字
	pub project_name: Vec<u8>,
	/// 代币名称
	pub symbol: Vec<u8>,
	/// logo地址
	pub logo_url: Vec<u8>,
	/// 项目是否做了认证
	pub is_identity: bool,
	/// 总发行量
	pub total_issuance: Balance,
	/// 总流通量
	pub total_circulation: Balance,
	/// 官网地址
	pub official_website: Vec<u8>,
	/// 公钥地址
	pub public_keys: Address,
	/// 用户参与的最小金额
	pub user_min_usdt: Option<USDT>,
	/// 用户参与的最大金额
	pub user_max_usdt: Option<USDT>,
	/// 这次募集资金拿出来的代币数量
	pub total_token_in_use: Balance,
	/// 募集资金的周期
	pub raise_duration: BlockNumber,
	/// 募集的usdt数量
	pub raise_usdt_total: USDT,
	/// 排除在外的国家
	pub exclude_countries: Vec<Vec<u8>>,
	/// 琐仓比例
	pub vesting_proportion: Percent,
	/// 单次解锁比例
	pub unlock_proportion: Percent,
}
