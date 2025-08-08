#[cfg(feature = "fvp")]
include!("../fvp/plat.rs");

#[cfg(feature = "qemu")]
include!("../qemu/plat.rs");
