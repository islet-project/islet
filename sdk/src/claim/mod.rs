pub mod platform;

use self::platform::SWComponent;
use crate::config;

pub type RealmSignature = Claim;
pub type RealmToken = [Claim; config::CLAIM_COUNT_REALM_TOKEN];
pub type PlatformSignature = Claim;
pub type PlatformToken = [Claim; config::CLAIM_COUNT_PLATFORM_TOKEN];
pub type PlatformSWComponents = [SWComponent; config::CLAIM_COUNT_SW_COMPONENT];

#[derive(Debug)]
pub enum Value {
    U16(u16),
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct Claim {
    pub label: u16,
    pub title: &'static str,
    pub value: Value,
}

#[derive(Debug)]
pub struct Claims {
    pub realm_sig: RealmSignature,
    pub realm_tok: RealmToken,
    pub plat_sig: PlatformSignature,
    pub plat_tok: PlatformToken,
    pub sw_comps: PlatformSWComponents, // TODO: Make to claim struct
}

impl Claims {
    pub fn get(&self, title: &'static str) -> Option<&Claim> {
        let title = Self::support_user_data(title);
        if title == self.realm_sig.title {
            return Some(&self.realm_sig);
        }

        for claim in &self.realm_tok {
            if title == claim.title {
                return Some(claim);
            }
        }

        if title == self.plat_sig.title {
            return Some(&self.realm_sig);
        }

        for claim in &self.plat_tok {
            if title == claim.title {
                return Some(claim);
            }
        }

        None
    }

    pub fn get_mut(&mut self, title: &'static str) -> Option<&mut Claim> {
        let title = Self::support_user_data(title);
        if title == self.realm_sig.title {
            return Some(&mut self.realm_sig);
        }

        for claim in &mut self.realm_tok {
            if title == claim.title {
                return Some(claim);
            }
        }

        if title == self.plat_sig.title {
            return Some(&mut self.realm_sig);
        }

        for claim in &mut self.plat_tok {
            if title == claim.title {
                return Some(claim);
            }
        }

        None
    }

    pub fn value(&self, title: &'static str) -> Option<&Value> {
        if title == config::STR_PLAT_SW_COMPONENTS {
            println!("Parsing claim[{}] is not supported yet.", title);
            return None;
        }
        Some(&self.get(title)?.value)
    }

    fn support_user_data(title: &'static str) -> &'static str {
        if title == config::STR_USER_DATA {
            config::STR_REALM_CHALLENGE
        } else {
            title
        }
    }
}
