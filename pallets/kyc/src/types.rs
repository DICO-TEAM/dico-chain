use codec::{Decode, Encode};
use sp_runtime::traits::{AppendZerosInput, Saturating, StaticLookup, Zero};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_std::{fmt::Debug, iter::once, ops::Add};

pub type KYCIndex = u32;
pub type ExchangeKey = [u8; 32];
pub type Message = [u8; 128];
pub type Data = Vec<u8>;


/// International Organization for Standardization (ISO)
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
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
