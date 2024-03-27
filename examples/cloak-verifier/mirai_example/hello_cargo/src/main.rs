#[macro_use]
extern crate mirai_annotations;

// MIRAI
#[cfg_attr(mirai, allow(incomplete_features), feature(generic_const_exprs))]

#[cfg(mirai)]
use mirai_annotations::{TagPropagation, TagPropagationSet};

#[cfg(mirai)]
struct TaintedKind<const MASK: TagPropagationSet> {}

#[cfg(mirai)]
const TAINTED_MASK: TagPropagationSet = tag_propagation_set!(TagPropagation::SubComponent);

#[cfg(mirai)]
type Tainted = TaintedKind<TAINTED_MASK>;  // Attach "Tainted" for secret
#[cfg(not(mirai))]
type Tainted = ();

#[cfg(mirai)]
struct SanitizedKind<const MASK: TagPropagationSet> {}

#[cfg(mirai)]
const SANITIZED_MASK: TagPropagationSet = tag_propagation_set!(TagPropagation::SubComponent);

#[cfg(mirai)]
type Sanitized = SanitizedKind<SANITIZED_MASK>;
#[cfg(not(mirai))]
type Sanitized = ();  // Attach "Sanitized" when secret is encrypted

#[derive(Clone, Copy)]
pub struct Data<S> {
    data: [u8; 4096],
    state: S,
}
pub struct None;
pub struct Unencrypted;
pub struct Encrypted;

pub trait DataState {
    fn dummy(&self) -> bool { true }
}
impl DataState for None {
    fn dummy(&self) -> bool { true }
}
impl DataState for Unencrypted {
    fn dummy(&self) -> bool { true }
}
impl DataState for Encrypted {
    fn dummy(&self) -> bool { true }
}

impl<S: DataState> Data<S> {
    pub fn read(data: [u8; 4096]) -> Data<Unencrypted> {
        let d = Data {
            data: data,
            state: Unencrypted,
        };
        add_tag!(&d, Tainted);
        d
    }
}
impl Data<Unencrypted> {
    pub fn encrypt(self) -> Data<Encrypted> {
        let d = Data {
            data: self.data,
            state: Encrypted
        };
        add_tag!(&d, Sanitized);
        d
    }
}

fn sink_func<S: DataState>(data: Data<S>) {
    precondition!(does_not_have_tag!(&data, Tainted) || has_tag!(&data, Sanitized));
    println!("hi");
}

fn mirai_test(data: &[u8; 4096]) {
    precondition!(does_not_have_tag!(data, Tainted) || has_tag!(data, Sanitized));
    println!("hi");
}

fn main() {
    let d: [u8; 4096] = [0; 4096];
    
    let data = Data::<None>::read(d);
    let data = data.encrypt();  // when it's commented out, MIRAI reported unsatisifed condition! (this matches my expectation)
    sink_func(data);

    /*
    // it seems that MIRAI is not capable of keeping track of a primitive type
    // i expected MIRAI to detect the below case as a unsatisfied precondtion, but MIRAI didn't.
    add_tag!(&d, Tainted);
    mirai_test(&d);
    */
    println!("Hello, world!");
}
