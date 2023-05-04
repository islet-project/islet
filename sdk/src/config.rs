pub const TOKEN_COUNT: u64 = 2;
pub const TOKEN_PLAT: u16 = 44234;
pub const TOKEN_REALM: u16 = 44241;

pub const CLAIM_COUNT_REALM_TOKEN: usize = 6;
pub const CLAIM_COUNT_PLATFORM_TOKEN: usize = 8;

pub const CHALLENGE_SIZE: usize = 64;

pub const TAG_CCA_TOKEN: u64 = 399;
pub const TAG_COSE_SIGN1: u64 = 18;

pub const TAG_REALM_CHALLENGE: u16 = 10;
pub const TAG_REALM_PERSONALIZATION_VALUE: u16 = 44235;
pub const TAG_REALM_HASH_ALGO_ID: u16 = 44240;
pub const TAG_REALM_PUB_KEY_HASH_ALGO_ID: u16 = 44236;
pub const TAG_REALM_PUB_KEY: u16 = 44237;
pub const TAG_REALM_INITIAL_MEASUREMENT: u16 = 44238;
pub const TAG_REALM_EXTENTIBLE_MEASUREMENTS: u16 = 44239;

pub const TAG_PLAT_CHALLENGE: u16 = 10;
pub const TAG_PLAT_VERIFICATION_SERVICE: u16 = 2400;
pub const TAG_PLAT_PROFILE: u16 = 265;
pub const TAG_PLAT_INSTANCE_ID: u16 = 256;
pub const TAG_PLAT_IMPLEMENTATION_ID: u16 = 2396;
pub const TAG_PLAT_SECURITY_LIFECYCLE: u16 = 2395;
pub const TAG_PLAT_CONFIGURATION: u16 = 2401;
pub const TAG_PLAT_HASH_ALGO_ID: u16 = 2402;
pub const TAG_PLAT_SW_COMPONENTS: u16 = 2399;
pub const TAG_UNASSIGINED: u16 = 0;

pub const STR_REALM_SIGNATURE: &str = "Realm Signature";
pub const STR_PLAT_SIGNATURE: &str = "Platform Signature";

pub const STR_USER_DATA: &str = "User data";
pub const STR_REALM_CHALLENGE: &str = "Realm challenge";
pub const STR_REALM_PERSONALIZATION_VALUE: &str = "Realm personalization value";
pub const STR_REALM_HASH_ALGO_ID: &str = "Realm hash algo id";
pub const STR_REALM_PUB_KEY_HASH_ALGO_ID: &str = "Realm public key hash algo id";
pub const STR_REALM_PUB_KEY: &str = "Realm signing public key";
pub const STR_REALM_INITIAL_MEASUREMENT: &str = "Realm initial measurement";
pub const STR_REALM_EXTENTIBLE_MEASUREMENTS: &str = "Realm extentible measurements";

pub const STR_PLAT_CHALLENGE: &str = "Challenge";
pub const STR_PLAT_VERIFICATION_SERVICE: &str = "Verification service";
pub const STR_PLAT_PROFILE: &str = "Profile";
pub const STR_PLAT_INSTANCE_ID: &str = "Instance ID";
pub const STR_PLAT_IMPLEMENTATION_ID: &str = "Implementation ID";
pub const STR_PLAT_SECURITY_LIFECYCLE: &str = "Lifecycle";
pub const STR_PLAT_CONFIGURATION: &str = "Configuration";
pub const STR_PLAT_HASH_ALGO_ID: &str = "Platform hash algo";
pub const STR_PLAT_SW_COMPONENTS: &str = "Platform sw components";

pub fn to_label(title: &'static str) -> u16 {
    match title {
        STR_USER_DATA | STR_REALM_CHALLENGE => TAG_REALM_CHALLENGE,
        STR_REALM_PERSONALIZATION_VALUE => TAG_REALM_PERSONALIZATION_VALUE,
        STR_REALM_HASH_ALGO_ID => TAG_REALM_HASH_ALGO_ID,
        STR_REALM_PUB_KEY_HASH_ALGO_ID => TAG_REALM_PUB_KEY_HASH_ALGO_ID,
        STR_REALM_PUB_KEY => TAG_REALM_PUB_KEY,
        STR_REALM_INITIAL_MEASUREMENT => TAG_REALM_INITIAL_MEASUREMENT,
        STR_REALM_EXTENTIBLE_MEASUREMENTS => TAG_REALM_EXTENTIBLE_MEASUREMENTS,
        STR_PLAT_CHALLENGE => TAG_PLAT_CHALLENGE,
        STR_PLAT_VERIFICATION_SERVICE => TAG_PLAT_VERIFICATION_SERVICE,
        STR_PLAT_PROFILE => TAG_PLAT_PROFILE,
        STR_PLAT_INSTANCE_ID => TAG_PLAT_INSTANCE_ID,
        STR_PLAT_IMPLEMENTATION_ID => TAG_PLAT_IMPLEMENTATION_ID,
        STR_PLAT_SECURITY_LIFECYCLE => TAG_PLAT_SECURITY_LIFECYCLE,
        STR_PLAT_CONFIGURATION => TAG_PLAT_CONFIGURATION,
        STR_PLAT_HASH_ALGO_ID => TAG_PLAT_HASH_ALGO_ID,
        STR_PLAT_SW_COMPONENTS => TAG_PLAT_SW_COMPONENTS,
        _ => TAG_UNASSIGINED,
    }
}
