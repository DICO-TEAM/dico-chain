#![feature(derive_default_enum)]

use codec::{Decode, Encode,MaxEncodedLen};
use sp_runtime::traits::{AppendZerosInput, Saturating, StaticLookup, Zero};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_std::{fmt::Debug, iter::once, ops::Add};
use scale_info::TypeInfo;

pub type KYCIndex = u32;
pub type CurvePubicKey = [u8; 32];
pub type Message = [u8; 128];
pub type Data = Vec<u8>;

/// IAS Judgement
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Judgement<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
	/// The default value; it has passed.
	PASS,
	/// No judgement is yet in place, but a deposit is reserved as payment for providing one.
	FeePaid(Balance),

	/// The data was once good but is currently out of date. There is no
	/// malicious intent in the inaccuracy. This judgement can be removed
	/// through updating the data.
	OutOfDate,
	/// The data is imprecise or of sufficiently low-quality to be problematic.
	/// It is not indicative of malicious intent. This judgement can be removed
	/// through updating the data.
	LowQuality,
	/// The data is erroneous. This may be indicative of malicious intent. This
	/// cannot be removed except by the registrar.
	Erroneous,
	/// The data is repeat. Maybe it was submitted before.
	/// By then the KYC user's ID number already exists
	Repeat,
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> Judgement<Balance> {
	/// Returns `true` if this judgement is indicative of a deposit being
	/// currently held.
	pub(crate) fn has_deposit(&self) -> bool {
		match self {
			Judgement::FeePaid(_) => true,
			_ => false,
		}
	}

	/// Returns `true` if this judgement is one that should not be generally be
	/// replaced outside of specialized handlers.
	pub(crate) fn is_sticky(&self) -> bool {
		match self {
			Judgement::FeePaid(_) | Judgement::Erroneous => true,
			_ => false,
		}
	}

	/// Returns `true` if this judgement has paid
	pub(crate) fn is_paid(&self) -> bool {
		match self {
			Judgement::FeePaid(_) => true,
			_ => false,
		}
	}
}

/// kyc information authentication by the `SwordHolder`
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Authentication {
	/// pending status
	Pending,
	/// Success： The SwordHolder believes that the certification is successful
	Success,
	/// Failure: The SwordHolder believes that the certification is not
	/// successful
	Failure,
}

impl Authentication {
	/// Certification passed
	pub(crate) fn has_success(&self) -> bool {
		match self {
			Authentication::Success => true,
			_ => false,
		}
	}

	/// Authentication failed
	pub(crate) fn has_failure(&self) -> bool {
		match self {
			Authentication::Failure => true,
			_ => false,
		}
	}

	pub(crate) fn is_pending(&self) -> bool {
		match self {
			Authentication::Pending => true,
			_ => false,
		}
	}
}

/// black info enum
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Black<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
	/// The default value; no opinion is held.
	Unknown,
	/// fraud
	Fraud(Balance),
	/// Information cheating
	Cheat,
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> Black<Balance> {
	/// cheat.
	pub(crate) fn is_cheat(&self) -> bool {
		match self {
			Black::Fraud(_) | Black::Cheat => true,
			_ => false,
		}
	}
}

/// The blacklist of kyc
#[derive(Clone, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BlackInfo<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
	pub info: Vec<Black<Balance>>,
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> Decode for BlackInfo<Balance> {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let info = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self { info })
	}
}

/// KYC field used by the user for authentication
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum KYCFields {
	Name,
	Area,
	CurvePubicKey,
	Email,
}

/// Information concerning the identity of the controller of an account.
///
/// NOTE: This should be stored at the end of the storage item to facilitate the
/// addition of extra fields in a backwards compatible way through a specialized
/// `Decode` impl.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct KYCInfo {
	/// display name
	/// Stored as UTF-8.
	pub name: Vec<u8>,

	/// The full area name
	/// Stored as UTF-8.
	pub area: AreaCode,

	/// public key to exchange
	/// Stored as UTF-8.
	pub curve_public_key: Vec<u8>,

	/// email
	/// Stored as UTF-8.
	pub email: Vec<u8>,
}

/// Information concerning the identity of the controller of an account.
#[derive(Clone, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Registration<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
	/// Judgements from the registrars on this identity. Stored ordered by
	pub judgements: Vec<(KYCFields, KYCIndex, Judgement<Balance>, Authentication)>,

	/// Amount held on deposit for this information.
	pub deposit: Balance,

	/// Information on the identity.
	pub info: KYCInfo,
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq + Zero + Add> Registration<Balance> {
	pub(crate) fn total_deposit(&self) -> Balance {
		self.deposit
			+ self
			.judgements
			.iter()
			.map(|(_, _, ref j, _)| {
				if let Judgement::FeePaid(fee) = j {
					*fee
				} else {
					Zero::zero()
				}
			})
			.fold(Zero::zero(), |a, i| a + i)
	}
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> Decode for Registration<Balance> {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let (judgements, deposit, info) = Decode::decode(&mut AppendZerosInput::new(input))?;
		Ok(Self {
			judgements,
			deposit,
			info,
		})
	}
}

/// Information concerning a identity authentication service(IAS).
#[derive(Clone, Encode, Decode, Eq, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct IASInfo<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	/// The account of the registrar.
	pub account: AccountId,

	/// Amount required to be given to the registrar for them to provide
	/// judgement.
	pub fee: Balance,

	/// public key to exchange
	/// Stored as UTF-8.
	pub curve_public_key: CurvePubicKey,

	/// Relevant fields for this registrar. Registrar judgements are limited to
	/// attestations on these fields.
	pub fields: KYCFields,
}

impl<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> IASInfo<Balance, AccountId>
{
	pub fn set_account(&mut self, account: AccountId) -> &mut Self {
		self.account = account;
		self
	}
	pub fn set_fee(&mut self, fee: Balance) -> &mut Self {
		self.fee = fee;
		self
	}

	pub fn set_curve_public_key(&mut self, curve_public_key: CurvePubicKey) -> &mut Self {
		self.curve_public_key = curve_public_key;
		self
	}

	pub fn set_fields(&mut self, fields: KYCFields) -> &mut Self {
		self.fields = fields;
		self
	}
}

/// ApplicationForm
#[derive(Clone, Encode, Decode, Eq, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ApplicationForm<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	/// The account of the registrar.
	pub ias: (KYCIndex, IASInfo<Balance, AccountId>),

	/// Amount required to be given to the registrar for them to provide
	/// judgement.
	pub sword_holder: (KYCIndex, IASInfo<Balance, AccountId>),

	/// Stored as UTF-8.
	pub progress: Progress,
}

impl<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> ApplicationForm<Balance, AccountId>
{
	pub fn set_ias(&mut self, index: KYCIndex, ias: IASInfo<Balance, AccountId>) -> &mut Self {
		self.ias = (index, ias);
		self
	}

	pub fn set_sword_holder(&mut self, index: KYCIndex, sword_holder: IASInfo<Balance, AccountId>) -> &mut Self {
		self.sword_holder = (index, sword_holder);
		self
	}

	pub fn set_progress(&mut self, progress: Progress) -> &mut Self {
		self.progress = progress;
		self
	}

	pub fn is_repeat(&self, kyc_fields: &KYCFields) -> bool {
		match self {
			Self {
				ias,
				sword_holder,
				progress,
			} => {
				// When Progress is Failure, can apply again.
				if ias.1.fields == kyc_fields.clone() && progress != &Progress::Failure {
					true
				} else {
					false
				}
			}
			_ => false,
		}
	}
}

/// Certification progress record
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Progress {
	/// pending status
	Pending,
	/// IAS Start this progress
	IasDoing,
	/// IAS Done this progress
	IasDone,
	/// sword_holder Done this progress
	SwordHolderDone,
	/// Success： The ias/SwordHolder believes that the certification is
	/// successful
	Success,
	/// Failure: The ias/SwordHolder believes that the certification is not
	/// successful
	Failure,
}

/// Record.
#[derive(Clone, Encode, Decode, Eq, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Record<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
	/// The account of the registrar.
	pub account: AccountId,

	/// Progress
	pub progress: Progress,

	/// KYCFields .
	pub fields: KYCFields,
}

/// International Organization for Standardization (ISO)
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum AreaCode {
	/// Afghanistan
	AF,
	/// Åland Islands
	AX,
	/// Albania
	AL,
	/// Algeria
	DZ,
	/// American Samoa
	AS,
	/// Andorra
	AD,
	/// Angola
	AO,
	/// Anguilla
	AI,
	/// Antarctica
	AQ,
	/// Antigua and Barbuda
	AG,
	/// Argentina
	AR,
	/// Armenia
	AM,
	/// Aruba
	AW,
	/// Australia
	AU,
	/// Austria
	AT,
	/// Azerbaijan
	AZ,
	/// Bahamas (the)
	BS,
	/// Bahrain
	BH,
	/// Bangladesh
	BD,
	/// Barbados
	BB,
	/// Belarus
	BY,
	/// Belgium
	BE,
	/// Belize
	BZ,
	/// Benin
	BJ,
	/// Bermuda
	BM,
	/// Bhutan
	BT,
	/// Bolivia (Plurinational State of)
	BO,
	/// Bonaire/Sint Eustatius/Saba
	BQ,
	/// Bosnia and Herzegovina
	BA,
	/// Botswana
	BW,
	/// Bouvet Island
	BV,
	/// Brazil
	BR,
	/// British Indian Ocean Territory (the)
	IO,
	/// Brunei Darussalam
	BN,
	/// Bulgaria
	BG,
	/// Burkina Faso
	BF,
	/// Burundi
	BI,
	/// Cabo Verde
	CV,
	/// Cambodia
	KH,
	/// Cameroon
	CM,
	/// Canada
	CA,
	/// Cayman Islands (the)
	KY,
	/// Central African Republic (the)
	CF,
	/// Chad
	TD,
	/// Chile
	CL,
	/// China
	CN,
	/// Christmas Island
	CX,
	/// Cocos (Keeling) Islands (the)
	CC,
	/// Colombia
	CO,
	/// Comoros (the)
	KM,
	/// Congo (the Democratic Republic of the)
	CD,
	/// Congo (the)
	CG,
	/// Cook Islands (the)
	CK,
	/// Costa Rica
	CR,
	/// Côte d'Ivoire
	CI,
	/// Croatia
	HR,
	/// Cuba
	CU,
	/// Curaçao
	CW,
	/// Cyprus
	CY,
	/// Czechia
	CZ,
	/// Denmark
	DK,
	/// Djibouti
	DJ,
	/// Dominica
	DM,
	/// Dominican Republic (the)
	DO,
	/// Ecuador
	EC,
	/// Egypt
	EG,
	/// El Salvador
	SV,
	/// Equatorial Guinea
	GQ,
	/// Eritrea
	ER,
	/// Estonia
	EE,
	/// Eswatini
	SZ,
	/// Ethiopia
	ET,
	/// Falkland Islands (the) [Malvinas]
	FK,
	/// Faroe Islands (the)
	FO,
	/// Fiji
	FJ,
	/// Finland
	FI,
	/// France
	FR,
	/// French Guiana
	GF,
	/// French Polynesia
	PF,
	///  French Southern Territories (the)
	TF,
	/// Gabon
	GA,
	/// Gambia (the)
	GM,
	/// Georgia
	GE,
	/// Germany
	DE,
	/// Ghana
	GH,
	/// Gibraltar
	GI,
	/// Greece
	GR,
	/// Greenland
	GL,
	/// Grenada
	GD,
	///  Guadeloupe
	GP,
	/// Guam
	GU,
	/// Guatemala
	GT,
	/// Guernsey
	GG,
	/// Guinea
	GN,
	/// Guinea-Bissau
	GW,
	/// Guyana
	GY,
	///  Haiti
	HT,
	/// Heard Island and McDonald Islands
	HM,
	/// Holy See (the)
	VA,
	///  Honduras
	HN,
	/// Hong Kong
	HK,
	/// Hungary
	HU,
	///  Iceland
	IS,
	/// India
	IN,
	/// Indonesia
	ID,
	/// Iran (Islamic Republic of)
	IR,
	///  Iraq
	IQ,
	/// Ireland
	IE,
	///  Isle of Man
	IM,
	/// Israel
	IL,
	///  Italy
	IT,
	/// Jamaica
	JM,
	/// Japan
	JP,
	/// Jersey
	JE,
	/// Jordan
	JO,
	/// Kazakhstan
	KZ,
	/// Kenya
	KE,
	/// Kiribati
	KI,
	/// Korea (the Democratic People's Republic of)
	KP,
	/// Korea (the Republic of)
	KR,
	/// Kuwait
	KW,
	/// Kyrgyzstan
	KG,
	///  Lao People's Democratic Republic (the)
	LA,
	///  Latvia
	LV,
	/// Lebanon
	LB,
	/// Lesotho
	LS,
	///  Liberia
	LR,
	/// Libya
	LY,
	/// Liechtenstein
	LI,
	///  Lithuania
	LT,
	/// Luxembourg
	LU,
	/// Macao
	MO,
	/// North Macedonia
	MK,
	/// Madagascar
	MG,
	/// Malawi
	MW,
	/// Malaysia
	MY,
	/// Maldives
	MV,
	/// Mali
	ML,
	/// Malta
	MT,
	/// Marshall Islands (the)
	MH,
	/// Martinique
	MQ,
	/// Mauritania
	MR,
	/// Mauritius
	MU,
	/// Mayotte
	YT,
	/// Mexico
	MX,
	/// Micronesia (Federated States of)
	FM,
	///  Moldova (the Republic of)
	MD,
	/// Monaco
	MC,
	/// Mongolia
	MN,
	/// Montenegro
	ME,
	/// Montserrat
	MS,
	/// Morocco
	MA,
	/// Mozambique
	MZ,
	/// Myanmar
	MM,
	/// Namibia
	NA,
	/// Nauru
	NR,
	/// Nepal
	NP,
	/// Netherlands (the)
	NL,
	/// New Caledonia
	NC,
	/// New Zealand
	NZ,
	/// Nicaragua
	NI,
	/// Niger (the)
	NE,
	/// Nigeria
	NG,
	/// Niue
	NU,
	/// Norfolk Island
	NF,
	/// Northern Mariana Islands (the)
	MP,
	/// Norway
	NO,
	/// Oman
	OM,
	/// Pakistan
	PK,
	/// Palau
	PW,
	/// Palestine, State of
	PS,
	/// Panama
	PA,
	/// Papua New Guinea
	PG,
	/// Paraguay
	PY,
	/// Peru
	PE,
	/// Philippines (the)
	PH,
	/// Pitcairn
	PN,
	/// Poland
	PL,
	/// Portugal
	PT,
	/// Puerto Rico
	PR,
	/// Qatar
	QA,
	/// Réunion
	RE,
	/// Romania
	RO,
	/// The Russian Federation
	RU,
	/// The Republic of Rwanda
	RW,
	/// The Collectivity of Saint-Barthélemy
	BL,
	/// Saint Helena, Ascension and Tristan da Cunha
	SH,
	/// Saint Kitts and Nevis
	KN,
	/// Saint Lucia
	LC,
	/// The Collectivity of Saint-Martin
	MF,
	/// The Overseas Collectivity of Saint-Pierre and Miquelon
	PM,
	/// Saint Vincent and the Grenadines
	VC,
	/// The Independent State of Samoa
	WS,
	/// The Republic of San Marino
	SM,
	/// The Democratic Republic of São Tomé and Príncipe
	ST,
	/// The Kingdom of Saudi Arabia
	SA,
	/// The Republic of Senegal
	SN,
	/// The Republic of Serbia
	RS,
	/// The Republic of Seychelles
	SC,
	/// The Republic of Sierra Leone
	SL,
	/// The Republic of Singapore
	SG,
	/// Sint Maarten
	SX,
	/// The Slovak Republic
	SK,
	/// The Republic of Slovenia
	SI,
	/// The Solomon Islands
	SB,
	///   The Federal Republic of Somalia
	SO,
	/// The Republic of South Africa
	ZA,
	/// South Georgia and the South Sandwich Islands
	GS,
	/// The Republic of South Sudan
	SS,
	/// The Kingdom of Spain
	ES,
	/// The Democratic Socialist Republic of Sri Lanka
	LK,
	/// The Republic of the Sudan
	SD,
	/// The Republic of Suriname
	SR,
	/// Svalbard and Jan Mayen
	SJ,
	/// The Kingdom of Sweden
	SE,
	/// The Swiss Confederation
	CH,
	/// The Syrian Arab Republic
	SY,
	/// Taiwan (Province of China)
	TW,
	/// The Republic of Tajikistan
	TJ,
	/// The United Republic of Tanzania
	TZ,
	/// The Kingdom of Thailand
	TH,
	/// The Democratic Republic of Timor-Leste
	TL,
	/// The Togolese Republic
	TG,
	/// Tokelau
	TK,
	/// The Kingdom of Tonga
	TO,
	/// The Republic of Trinidad and Tobago
	TT,
	/// The Republic of Tunisia
	TN,
	/// The Republic of Turkey
	TR,
	/// Turkmenistan
	TM,
	/// The Turks and Caicos Islands
	TC,
	/// Tuvalu
	TV,
	/// The Republic of Uganda
	UG,
	/// Ukraine
	UA,
	/// The United Arab Emirates
	AE,
	/// The United Kingdom of Great Britain and Northern Ireland
	GB,
	/// Baker Island, Howland Island, Jarvis Island, Johnston Atoll, Kingman
	/// Reef, Midway Atoll, Navassa Island, Palmyra Atoll, and Wake Island
	UM,
	/// The United States of America
	US,
	/// The Oriental Republic of Uruguay
	UY,
	/// The Republic of Uzbekistan
	UZ,
	/// The Republic of Vanuatu
	VU,
	/// The Bolivarian Republic of Venezuela
	VE,
	/// The Socialist Republic of Viet Nam
	VN,
	/// The Virgin Islands
	VG,
	/// The Virgin Islands of the United States
	VI,
	/// The Territory of the Wallis and Futuna Islands
	WF,
	/// The Sahrawi Arab Democratic Republic
	EH,
	/// The Republic of Yemen
	YE,
	/// The Republic of Zambia
	ZM,
	/// The Republic of Zimbabwe
	ZW,
}

impl AreaCode {
	pub fn is_illegal(&self) -> bool {
		match self {
			AreaCode::DZ | AreaCode::EG | AreaCode::MA | AreaCode::BO | AreaCode::NP => true,
			_ => false,
		}
	}
	pub fn is_banking_ban(&self) -> bool {
		match self {
			AreaCode::CN
			| AreaCode::NG
			| AreaCode::CA
			| AreaCode::CO
			| AreaCode::EC
			| AreaCode::RU
			| AreaCode::SA
			| AreaCode::JO
			| AreaCode::TR
			| AreaCode::QA
			| AreaCode::IR
			| AreaCode::BD
			| AreaCode::NP
			| AreaCode::CN
			| AreaCode::TW
			| AreaCode::KH
			| AreaCode::ID
			| AreaCode::VN => true,
			_ => false,
		}
	}
}