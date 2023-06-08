pub(crate) mod dumper;
pub(crate) mod verifier;

use ciborium::de;
use coset::CoseSign1;
use std::default::Default;
use std::fmt::Debug;

const TAG_COSE_SIGN1: u64 = 18;
const TAG_CCA_TOKEN: u64 = 399;

const CCA_PLAT_TOKEN: u32 = 44234;
const CCA_REALM_DELEGATED_TOKEN: u32 = 44241;

/* CCA Platform Attestation Token */
const CCA_PLAT_CHALLENGE: u32 = 10;
const CCA_PLAT_INSTANCE_ID: u32 = 256;
const CCA_PLAT_PROFILE: u32 = 265;
const CCA_PLAT_SECURITY_LIFECYCLE: u32 = 2395;
const CCA_PLAT_IMPLEMENTATION_ID: u32 = 2396;
const CCA_PLAT_SW_COMPONENTS: u32 = 2399;
const CCA_PLAT_VERIFICATION_SERVICE: u32 = 2400;
const CCA_PLAT_CONFIGURATION: u32 = 2401;
const CCA_PLAT_HASH_ALGO_ID: u32 = 2402;

/* CCA Realm Delegated Attestation Token */
const CCA_REALM_CHALLENGE: u32 = 10;
const CCA_REALM_PERSONALIZATION_VALUE: u32 = 44235;
const CCA_REALM_HASH_ALGO_ID: u32 = 44236;
const CCA_REALM_PUB_KEY: u32 = 44237;
const CCA_REALM_INITIAL_MEASUREMENT: u32 = 44238;
const CCA_REALM_EXTENSIBLE_MEASUREMENTS: u32 = 44239;
const CCA_REALM_PUB_KEY_HASH_ALGO_ID: u32 = 44240;

/* Software components */
const CCA_SW_COMP_TITLE: u32 = 1;
const CCA_SW_COMP_MEASUREMENT_VALUE: u32 = 2;
const CCA_SW_COMP_VERSION: u32 = 4;
const CCA_SW_COMP_SIGNER_ID: u32 = 5;
const CCA_SW_COMP_HASH_ALGORITHM: u32 = 6;

/* Counts */
const CLAIM_COUNT_REALM_TOKEN: usize = 6;
const CLAIM_COUNT_COSE_SIGN1_WRAPPER: usize = 3;
const CLAIM_COUNT_PLATFORM_TOKEN: usize = 8;
const CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS: usize = 4;
const CLAIM_COUNT_SW_COMPONENT: usize = 5;
const MAX_SW_COMPONENT_COUNT: usize = 32;

#[derive(Debug)]
pub enum ClaimData
{
    Bool(bool),
    Int64(i64),
    Bstr(Vec<u8>),
    Text(String),
}

#[allow(dead_code)]
impl ClaimData
{
    fn new_bool() -> Self
    {
        ClaimData::Bool(false)
    }
    fn new_int64() -> Self
    {
        ClaimData::Int64(0)
    }
    fn new_bstr() -> Self
    {
        ClaimData::Bstr(Vec::new())
    }
    fn new_text() -> Self
    {
        ClaimData::Text(String::new())
    }

    fn get_bool(&self) -> bool
    {
        if let ClaimData::Bool(b) = self {
            return *b;
        } else {
            panic!("ClaimData is not Bool");
        }
    }
    fn get_int64(&self) -> i64
    {
        if let ClaimData::Int64(i) = self {
            return *i;
        } else {
            panic!("ClaimData is not Int64");
        }
    }
    fn get_bstr(&self) -> &[u8]
    {
        if let ClaimData::Bstr(d) = self {
            return d;
        } else {
            panic!("ClaimData is not Bstr");
        }
    }
    fn get_text(&self) -> &str
    {
        if let ClaimData::Text(s) = self {
            return s;
        } else {
            panic!("ClaimData is not Text");
        }
    }
}

impl Default for ClaimData
{
    fn default() -> Self
    {
        Self::Bool(false)
    }
}

#[derive(Debug, Default)]
pub struct Claim
{
    pub mandatory: bool,
    pub key: i64,
    pub title: String,
    pub present: bool,
    pub data: ClaimData,
}

impl Claim
{
    fn init<T>(&mut self, mandatory: bool, data: ClaimData, key: T, title: &str, present: bool)
    where
        T: Into<i64>,
    {
        self.mandatory = mandatory;
        self.data = data;
        self.key = key.into();
        self.title = title.to_string();
        self.present = present;
    }
}

#[derive(Debug, Default)]
pub struct SwComponent
{
    pub present: bool,
    pub claims: [Claim; CLAIM_COUNT_SW_COMPONENT],
}

#[derive(Debug, Default)]
pub struct AttestationClaims
{
    pub realm_cose_sign1_wrapper: [Claim; CLAIM_COUNT_COSE_SIGN1_WRAPPER],
    pub realm_cose_sign1: CoseSign1,
    pub realm_token_claims: [Claim; CLAIM_COUNT_REALM_TOKEN],
    pub realm_measurement_claims: [Claim; CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS],
    pub plat_cose_sign1_wrapper: [Claim; CLAIM_COUNT_COSE_SIGN1_WRAPPER],
    pub plat_cose_sign1: CoseSign1,
    pub plat_token_claims: [Claim; CLAIM_COUNT_PLATFORM_TOKEN],
    pub sw_component_claims: [SwComponent; MAX_SW_COMPONENT_COUNT],
}

impl AttestationClaims
{
    fn init_cose_sign1_claims(wrapper: &mut [Claim; CLAIM_COUNT_COSE_SIGN1_WRAPPER])
    {
        wrapper[0].init(true, ClaimData::new_bstr(), 0, "Protected header", false);
        wrapper[1].init(true, ClaimData::new_bstr(), 0, "Token payload", false);
        wrapper[2].init(true, ClaimData::new_bstr(), 0, "Signature", false);
    }

    pub(crate) fn new() -> Self
    {
        let mut claims = Self::default();

        Self::init_cose_sign1_claims(&mut claims.realm_cose_sign1_wrapper);

        claims.realm_token_claims[0].init(
            true,
            ClaimData::new_bstr(),
            CCA_REALM_CHALLENGE,
            "Realm challenge",
            false,
        );
        claims.realm_token_claims[1].init(
            true,
            ClaimData::new_bstr(),
            CCA_REALM_PERSONALIZATION_VALUE,
            "Realm personalization value",
            false,
        );
        claims.realm_token_claims[2].init(
            true,
            ClaimData::new_text(),
            CCA_REALM_HASH_ALGO_ID,
            "Realm hash algo id",
            false,
        );
        claims.realm_token_claims[3].init(
            true,
            ClaimData::new_text(),
            CCA_REALM_PUB_KEY_HASH_ALGO_ID,
            "Realm public key hash algo id",
            false,
        );
        claims.realm_token_claims[4].init(
            true,
            ClaimData::new_bstr(),
            CCA_REALM_PUB_KEY,
            "Realm signing public key",
            false,
        );
        claims.realm_token_claims[5].init(
            true,
            ClaimData::new_bstr(),
            CCA_REALM_INITIAL_MEASUREMENT,
            "Realm initial measurement",
            false,
        );

        Self::init_cose_sign1_claims(&mut claims.plat_cose_sign1_wrapper);

        claims.plat_token_claims[0].init(
            true,
            ClaimData::new_bstr(),
            CCA_PLAT_CHALLENGE,
            "Challange",
            false,
        );
        claims.plat_token_claims[1].init(
            false,
            ClaimData::new_text(),
            CCA_PLAT_VERIFICATION_SERVICE,
            "Verification service",
            false,
        );
        claims.plat_token_claims[2].init(
            true,
            ClaimData::new_text(),
            CCA_PLAT_PROFILE,
            "Profile",
            false,
        );
        claims.plat_token_claims[3].init(
            true,
            ClaimData::new_bstr(),
            CCA_PLAT_INSTANCE_ID,
            "Instance ID",
            false,
        );
        claims.plat_token_claims[4].init(
            true,
            ClaimData::new_bstr(),
            CCA_PLAT_IMPLEMENTATION_ID,
            "Implementation ID",
            false,
        );
        claims.plat_token_claims[5].init(
            true,
            ClaimData::new_int64(),
            CCA_PLAT_SECURITY_LIFECYCLE,
            "Lifecycle",
            false,
        );
        claims.plat_token_claims[6].init(
            true,
            ClaimData::new_bstr(),
            CCA_PLAT_CONFIGURATION,
            "Configuration",
            false,
        );
        claims.plat_token_claims[7].init(
            true,
            ClaimData::new_text(),
            CCA_PLAT_HASH_ALGO_ID,
            "Platform hash algo",
            false,
        );

        let mut count = 0;
        for claim in &mut claims.realm_measurement_claims {
            claim.init(
                true,
                ClaimData::new_bstr(),
                count,
                "Realm extensible measurement",
                false,
            );
            count += 1;
        }

        for component in &mut claims.sw_component_claims {
            component.present = false;
            component.claims[0].init(
                true,
                ClaimData::new_text(),
                CCA_SW_COMP_TITLE,
                "SW Type",
                false,
            );
            component.claims[1].init(
                false,
                ClaimData::new_text(),
                CCA_SW_COMP_HASH_ALGORITHM,
                "Hash algorithm",
                false,
            );
            component.claims[2].init(
                true,
                ClaimData::new_bstr(),
                CCA_SW_COMP_MEASUREMENT_VALUE,
                "Measurement value",
                false,
            );
            component.claims[3].init(
                false,
                ClaimData::new_text(),
                CCA_SW_COMP_VERSION,
                "Version",
                false,
            );
            component.claims[4].init(
                true,
                ClaimData::new_bstr(),
                CCA_SW_COMP_SIGNER_ID,
                "Signer ID",
                false,
            );
        }

        claims
    }
}

#[derive(Debug)]
pub enum TokenError
{
    InvalidKey(&'static str),
    InvalidTag(&'static str),
    InvalidTokenFormat(&'static str),
    NotImplemented(&'static str),
    InvalidAlgorithm(Option<coset::Algorithm>),
    Signature,
    Ciborium(de::Error<std::io::Error>),
    Coset(coset::CoseError),
    Ecdsa(ecdsa::Error),
}

impl std::fmt::Display for TokenError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TokenError {}

impl From<de::Error<std::io::Error>> for TokenError
{
    fn from(value: de::Error<std::io::Error>) -> Self
    {
        Self::Ciborium(value)
    }
}

impl From<coset::CoseError> for TokenError
{
    fn from(value: coset::CoseError) -> Self
    {
        Self::Coset(value)
    }
}

impl From<ecdsa::Error> for TokenError
{
    fn from(value: ecdsa::Error) -> Self
    {
        Self::Ecdsa(value)
    }
}
