//! # KYC Module
//!
//! ## Overview
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users (KYC用户)  属性提供者
//! * `set_kyc` - Set the associated kyc of an account; a small deposit is reserved if not
//!   already taken.
//! * `clear_kyc` - Remove an account's associated kyc; the deposit is returned.
//! * `request_judgement` - Request a judgement from a registrar, paying a fee.
//! * `cancel_request` - Cancel the previous request for a judgement.
//!
//! #### For judge(身份认证服务商IAS) --> 属性验证者
//!
//! * `judge_set_fee` - Set the fee required to be paid for a judgement to be given by the judge.
//! * `judge_set_fields` - Set the fields that a registrar cares about in their judgements.
//! * `judge_provide_judgement` - Provide a judgement to an kyc account.
//!
//! #### For supervision(监管)  --> 属性监管者
//!
//! * `supervisor_provide_judgement` - Provide a judgement to an kyc account.
//! * `apply_permission` - apply_permission
//!
//! #### For sudo super-users(Sudo)
//! * `add_judge` - Add a new registrar to the system. TODO: Formed by election
//! * `add_supervisor` - Add a new registrar to the system. TODO: Formed by election
//! * `remove_kyc` - remote kyc
//! * `kill_judge` - Forcibly remove the associated judge; the deposit is lost.



#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::{fmt::Debug, ops::Add, iter::once};
use enumflags2::BitFlags;
use codec::{Encode, Decode};
use sp_runtime::{DispatchError, RuntimeDebug, DispatchResult};
use sp_runtime::traits::{StaticLookup, Zero, AppendZerosInput, Saturating};
use frame_support::{
    decl_module, decl_event, decl_storage, ensure, decl_error,
    dispatch::DispatchResultWithPostInfo,
    traits::{Currency, ReservableCurrency, OnUnbalanced, Get, BalanceStatus, EnsureOrigin},
    weights::Weight,
};
use frame_system::ensure_signed;


type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub type JudgeIndex = u32;
pub type SupervisorIndex = u32;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type BasicDeposit: Get<BalanceOf<Self>>;

    type FieldDeposit: Get<BalanceOf<Self>>;

    type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

    type ForceOrigin: EnsureOrigin<Self::Origin>;

    type JudgeOrigin: EnsureOrigin<Self::Origin>;

    type SupervisorOrigin: EnsureOrigin<Self::Origin>;
}


#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Data {
    /// No data here.
    None,
    /// The data is stored directly.
    Raw(Vec<u8>),
    /// Only the Blake2 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    BlakeTwo256([u8; 32]),
    /// Only the SHA2-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Sha256([u8; 32]),
    /// Only the Keccak-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Keccak256([u8; 32]),
    /// Only the SHA3-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    ShaThree256([u8; 32]),
}

// codec  如果其他的进行转换要转化为u8,需要对下面进行重构
impl Decode for Data {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        //  如果是其他非u8进制，要让代码进行 转换为 u8
        let b = input.read_byte()?;
        Ok(match b {
            0 => Data::None,
            n @ 1..=33 => {
                let mut r = vec![0u8; n as usize - 1];
                input.read(&mut r[..])?;
                Data::Raw(r)
            }
            34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
            35 => Data::Sha256(<[u8; 32]>::decode(input)?),
            36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
            37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
            _ => return Err(codec::Error::from("invalid leading byte")),
        })
    }
}

impl Encode for Data {
    fn encode(&self) -> Vec<u8> {
        match self {
            Data::None => vec![0u8; 1],
            Data::Raw(ref x) => {
                let l = x.len().min(32);
                let mut r = vec![l as u8 + 1; l + 1];
                &mut r[1..].copy_from_slice(&x[..l as usize]);
                r
            }
            Data::BlakeTwo256(ref h) => once(34u8).chain(h.iter().cloned()).collect(),
            Data::Sha256(ref h) => once(35u8).chain(h.iter().cloned()).collect(),
            Data::Keccak256(ref h) => once(36u8).chain(h.iter().cloned()).collect(),
            Data::ShaThree256(ref h) => once(37u8).chain(h.iter().cloned()).collect(),
        }
    }
}

impl codec::EncodeLike for Data {}

impl Default for Data {
    fn default() -> Self {
        Self::None
    }
}


// 评审结果
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub enum Judgement<
    Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq
> {
    /// 未知，默认
    Unknown,
    /// 评审中，保留提交的费用
    FeePaid(Balance),
    /// 过期
    OutOfDate,
    /// 资料未齐全
    IncompleteInformation,

    /// 资料未达标
    NotRequiredData,

    /// 可以信赖的
    Trustworthy,

    /// 资料错误
    Erroneous,

    /// 作弊
    Cheat,
}


impl<
    Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq
> Judgement<Balance> {
    fn has_deposit(&self) -> bool {
        match self {
            Judgement::FeePaid(_) => true,
            _ => false,
        }
    }

    fn is_sticky(&self) -> bool {
        match self {
            Judgement::FeePaid(_) | Judgement::Erroneous => true,
            _ => false,
        }
    }

    fn is_cheat(&self) -> bool {
        match self {
            Judgement::FeePaid(_) | Judgement::Cheat => true,
            _ => false,
        }
    }
}


#[repr(u64)]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, BitFlags, RuntimeDebug)]
pub enum KycField {
    Name = 1 << 0,
    Country = 1 << 1,
    Web = 1 << 2,
    Email = 1 << 3,
    Github = 1 << 4,
    Twitter = 1 << 5,
    Info = 1 << 6,
    PgpPublicKey = 1 << 7, // dico 命令行 生成
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
pub struct KycFields(BitFlags<KycField>);

impl Eq for KycFields {}

// Encode 和 Decode 编码和解码最小单位是 u8, 更大单位要进行转换
impl Encode for KycFields {
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        self.0.bits().using_encoded(f)
    }
}

impl Decode for KycFields {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let field = u64::decode(input)?;
        Ok(Self(<BitFlags<KycField>>::from_bits(field as u64).map_err(|_| "invalid value")?))
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(test, derive(Default))]
pub struct KYCInfo {
    /// 需要增加部分  -> 数据部分，dico 命令行生成
    pub additional: Vec<(Data, Data)>,

    /// 社交信息  -> 数据部分，dico 命令行生成
    pub social: Vec<(Data, Data)>,

    /// 需要验证资料  -> 数据部分，dico 命令行生成
    pub validation_data: Vec<(Data, Data)>,

    /// 名字
    pub name: Data,

    /// 网站
    pub web: Data,

    /// email
    pub email: Data,

    /// github
    pub github: Data,

    /// twitter
    pub twitter: Data,

    /// 信息说明,可以时候一个地址 -> 数据部分，dico 命令行生成
    pub info: Data,

    /// pgp public key ，dico 命令行生成
    pub pgp_pub: Option<[u8; 20]>,

}

//  Judge 的使用事项
#[derive(Clone, Encode, Eq, PartialEq, RuntimeDebug)]
pub struct Judge<
    Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq
> {
    pub judgements: Vec<(JudgeIndex, Judgement<Balance>)>,

    pub deposit: Balance,

    pub info: KYCInfo,

}

impl<
    Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq + Zero + Add,
> Judge<Balance> {
    fn total_deposit(&self) -> Balance {
        self.deposit + self.judgements.iter()
            .map(|(_, ref j)| if let Judgement::FeePaid(fee) = j { *fee } else { Zero::zero() })
            .fold(Zero::zero(), |a, i| a + i)
    }
}

impl<
    Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq,
> Decode for Judge<Balance> {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let (judgements, deposit, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
        Ok(Self { judgements, deposit, info })
    }
}


//  supervisor 使用事项
#[derive(Clone, Encode, Eq, PartialEq, RuntimeDebug)]
pub struct Supervisor<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
    pub judgements: Vec<(JudgeIndex, Judgement<Balance>)>,

    // 如果发现判断错误，那么Judge 的钱会减少，然后给Supervisor
    pub reward: Balance,

    pub info: KYCInfo,

}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq, > Decode for Supervisor<Balance> {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let (judgements, reward, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
        Ok(Self { judgements, reward, info })
    }
}


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct JudgeInfo<
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq
> {
    pub account: AccountId,

    pub fee: Balance,

    pub fields: KycFields,
}

decl_storage! {
    trait Store for Module<T: Trait> as Kyc {
        pub KYCOf get(fn kyc):
            map hasher(twox_64_concat) T::AccountId => Option<Judge<BalanceOf<T>>>;

        pub Judges get(fn judges): Vec<Option<JudgeInfo<BalanceOf<T>, T::AccountId>>>;
        // 存储结构 和 Judges 存储结构一样
        pub Supervisors get(fn supervisors): Vec<Option<JudgeInfo<BalanceOf<T>, T::AccountId>>>;
    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, Balance = BalanceOf<T> {

        JudgeAdded(JudgeIndex),
        SupervisorAdded(SupervisorIndex),
        KYCSet(AccountId),
        KYCCleared(AccountId, Balance),
        JudgementRequested(AccountId, JudgeIndex),
        CanelJudgementRequested(AccountId, JudgeIndex),
        RemoveKYC(AccountId, Balance)


    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Account isn't found.
        NotFound,
        /// Account isn't named.
        NotNamed,
        /// EmptyIndex
        EmptyIndex,
        /// InvalidIndex
        InvalidIndex,
        JudgementGiven,

        NotEnoughFee,
        StickyJudgement,
        CheatJudgement,
        InvalidTarget,
        InvalidJudgement,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        const BasicDeposit: BalanceOf<T> = T::BasicDeposit::get();

        const FieldDeposit: BalanceOf<T> = T::FieldDeposit::get();


        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 120_000_000]
        fn add_judge(origin, account: T::AccountId) -> DispatchResult {
            T::JudgeOrigin::ensure_origin(origin)?;

            let (i, judge_count) = <Judges<T>>::try_mutate(
                |judges| -> Result<(JudgeIndex, usize), DispatchError> {
                    // TODO: 增加judge的限制, 选举或者说某一方面的限制
                    judges.push(Some(JudgeInfo {
                        account, fee: Zero::zero(), fields: Default::default()
                    }));
                    Ok(((judges.len() - 1) as JudgeIndex, judges.len()))
                }
            )?;

            Self::deposit_event(RawEvent::JudgeAdded(i));

            Ok(())
        }

        /// 一个机构存在之后，可以发送成为这个机构的 监管者
        #[weight = 120_000_000]
        fn apply_supervisor(origin, account: T::AccountId) -> DispatchResult {

            T::SupervisorOrigin::ensure_origin(origin)?;

            let (i, supervisor_count) = <Supervisors<T>>::try_mutate(
                |supervisors| -> Result<(SupervisorIndex, usize), DispatchError> {
                    // 和 judge 存储结构一样
                    // TODO: 对每个发证的机构进行最大监控者的限制
                    supervisors.push(Some(JudgeInfo {
                        account, fee: Zero::zero(), fields: Default::default()
                    }));
                    Ok(((judges.len() - 1) as JudgeIndex, judges.len()))
                }
            )?;

            Self::deposit_event(RawEvent::SupervisorAdded(i));

            Ok(())
        }


        #[weight = 120_000_000]
        fn set_kyc(origin, info: KYCInfo) -> DispatchResult {
            // TODO: 自定义 KYCInfo 的数据结构
            let sender = ensure_signed(origin)?;
            // TODO: 设置KYC额外信息的限制
            let extra_fields = info.additional.len() as u32;

            let mut id = match <KYCOf<T>>::get(&sender) {
                Some(mut id) => {
                    id.judgements.retain(|j| j.1.is_sticky());
                    id.info = info;
                    id
                }
                None => Judge { info, judgements: Vec::new(), deposit: Zero::zero() },
            };

            // deposit 调整
            let old_deposit = id.deposit;
            id.deposit = T::BasicDeposit::get();
            if id.deposit > old_deposit {
                T::Currency::reserve(&sender, id.deposit - old_deposit)?;
            }
            if old_deposit > id.deposit {
                let _ = T::Currency::unreserve(&sender, old_deposit - id.deposit);
            }

            let judgements = id.judgements.len();
            <KYCOf<T>>::insert(&sender, id);
            Self::deposit_event(RawEvent::KYCSet(sender));

            // TODO: 调整返回结果
            Ok(())
        }

        #[weight = 120_000_000]
        fn clear_kyc(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let id = <KYCOf<T>>::take(&sender).ok_or(Error::<T>::NotNamed)?;
            let deposit = id.total_deposit();

            let _ = T::Currency::unreserve(&sender, deposit.clone());

            Self::deposit_event(RawEvent::KYCCleared(sender, deposit));

            Ok(())
        }

        // 发送验证请求
        #[weight = 120_000_000]
        fn request_judgement(origin,
            #[compact] judge_index: JudgeIndex,
            #[compact] max_fee: BalanceOf<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let judges = <Judges<T>>::get();
            let judge = judges.get(judge_index as usize).and_then(Option::as_ref)
                .ok_or(Error::<T>::EmptyIndex)?;
            ensure!(max_fee >= judge.fee, Error::<T>::NotEnoughFee);

            let mut id = <KYCOf<T>>::get(&sender).ok_or(Error::<T>::NotFound)?;

            let item = (judge_index, Judgement::FeePaid(judge.fee));
            match id.judgements.binary_search_by_key(&judge_index, |x| x.0) {
                Ok(i) => if id.judgements[i].1.is_sticky() {
                    Err(Error::<T>::StickyJudgement)?
                } else if id.judgements[i].1.is_cheat() {
                    Err(Error::<T>::CheatJudgement)?
                } else {
                    id.judgements[i] = item
                },
                Err(i) => id.judgements.insert(i, item),
            }

            // 锁定费用
            T::Currency::reserve(&sender, judge.fee)?;

            <KYCOf<T>>::insert(&sender, id);

            Self::deposit_event(RawEvent::JudgementRequested(sender, judge_index));

            // TODO: 调整返回结果
            Ok(())
        }


        #[weight = 120_000_000]
        fn cancel_request(origin, jugle_index: JudgeIndex) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let mut id = <KYCOf<T>>::get(&sender).ok_or(Error::<T>::NotFound)?;

            let pos = id.judgements.binary_search_by_key(&jugle_index, |x| x.0)
                .map_err(|_| Error::<T>::NotFound)?;

            let fee = if let Judgement::FeePaid(fee) = id.judgements.remove(pos).1 {
                fee
            } else {
                Err(Error::<T>::JudgementGiven)?
            };

            // 解除锁仓
            let _ = T::Currency::unreserve(&sender, fee);

            <KYCOf<T>>::insert(&sender, id);

            Self::deposit_event(RawEvent::CanelJudgementRequested(sender, jugle_index));

            Ok(())
        }

        /// judge set fee
        #[weight = 120_000_000]
        fn judge_set_fee(origin,
            #[compact] index: judge_index,
            #[compact] fee: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let judges = <Judges<T>>::mutate(|j| -> Result<usize, DispatchError> {
                j.get_mut(index as usize)
                    .and_then(|x| x.as_mut())
                    .and_then(|r| if r.account == who { r.fee = fee; Some(()) } else { None })
                    .ok_or_else(|| DispatchError::from(Error::<T>::InvalidIndex))?;
                Ok(rs.len())
            })?;
            Ok(())
        }


        /// judge 设置费用
        #[weight = 120_000_000]
        fn judge_set_fields(origin,
            #[compact] index: JudgeIndex,
            fields: KycFields,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let judges = <Judges<T>>::mutate(|j| -> Result<usize, DispatchError> {
                j.get_mut(index as usize)
                    .and_then(|x| x.as_mut())
                    .and_then(|r| if r.account == who { r.fields = fields; Some(()) } else { None })
                    .ok_or_else(|| DispatchError::from(Error::<T>::InvalidIndex))?;
                Ok(rs.len())
            })?;
            Ok(())
        }


        #[weight = 120_000_000]
        fn judge_provide_judgement(origin,
            #[compact] judge_index: JudgeIndex,
            target: <T::Lookup as StaticLookup>::Source,
            judgement: Judgement<BalanceOf<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;
            ensure!(!judgement.has_deposit(), Error::<T>::InvalidJudgement);

            <Judges<T>>::get()
                .get(judge_index as usize)
                .and_then(Option::as_ref)
                .and_then(|j| if r.account == sender { Some(j) } else { None })
                .ok_or(Error::<T>::InvalidIndex)?;
            let mut id = <KYCOf<T>>::get(&target).ok_or(Error::<T>::InvalidTarget)?;

            let item = (reg_index, judgement);
            match id.judgements.binary_search_by_key(&judge_index, |x| x.0) {
                Ok(position) => {
                    if let Judgement::FeePaid(fee) = id.judgements[position].1 {
                        let _ = T::Currency::repatriate_reserved(&target, &sender, fee, BalanceStatus::Free);
                    }
                    id.judgements[position] = item
                }
                Err(position) => id.judgements.insert(position, item),
            }

            <KYCOf<T>>::insert(&target, id);
            Self::deposit_event(RawEvent::JudgementGiven(target, reg_index));
            Ok(())
        }

        /// 监管者实现监管逻辑，需要获得查看权
        #[weight = 120_000_000]
        fn supervisor_provide_judgement(origin,
            #[compact] judge_index: JudgeIndex,
            target: <T::Lookup as StaticLookup>::Source,
            // supervisor 一旦设定某个阶段，那么会改变之前的评判结果
            judgement: Judgement<BalanceOf<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;
            ensure!(!judgement.has_deposit(), Error::<T>::InvalidJudgement);


            // 密码转移权限，TODO: 如果实现,那么下面逻辑就没有问题
            <Judges<T>>::get()
                .get(judge_index as usize)
                .and_then(Option::as_ref)
                .and_then(|j| if j.account == sender { Some(j) } else { None })
                .ok_or(Error::<T>::InvalidIndex)?;
            let mut id = <KYCOf<T>>::get(&target).ok_or(Error::<T>::InvalidTarget)?;


            let item = (judge_index, judgement);
            match id.judgements.binary_search_by_key(&judge_index, |x| x.0) {
                Ok(position) => {
                    if let Judgement::FeePaid(fee) = id.judgements[position].1 {
                        let _ = T::Currency::repatriate_reserved(&target, &sender, fee, BalanceStatus::Free);
                    }
                    id.judgements[position] = item
                }
                Err(position) => id.judgements.insert(position, item),
            }

            <KYCOf<T>>::insert(&target, id);
            Self::deposit_event(RawEvent::JudgementGiven(target, reg_index));
            Ok(())
        }

        // 当用户不想要某个身份的时候，可以直接解除，费用不退
        #[weight = 120_000_000]
        fn remove_kyc(origin, target: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;

            // Figure out who we're meant to be clearing.
            let target = T::Lookup::lookup(target)?;
            // Grab their deposit (and check that they have one).
            let id = <KYCOf<T>>::take(&target).ok_or(Error::<T>::NotFound)?;
            let deposit = id.total_deposit();

            Self::deposit_event(RawEvent::RemoveKYC(target, deposit));

            Ok(())
        }



    }
}

impl<T: Trait> Module<T> {
    /// Transfer ownership
    fn _transfer(sender: T::AccountId, receiver: T::AccountId, #[compact] judge_index: JudgeIndex) -> DispatchResult {
        let judge = <Judges<T>>::get()
            .get(judge_index as usize)
            .and_then(Option::as_ref)
            .and_then(|j| if j.account == sender { Some(j) } else { None })
            .ok_or(Error::<T>::InvalidIndex)?;

        // TODO: 对judge 信息进行解密

        Ok(())
    }

    fn _copy(sender: T::AccountId, receiver: T::AccountId, #[compact] judge_index: JudgeIndex) -> DispatchResult {
        // TODO: 实现加密解密的数据传递
        Ok(())
    }
}
