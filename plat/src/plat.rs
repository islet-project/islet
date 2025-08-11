#[cfg(any(feature = "fvp", not(feature = "qemu")))]
include!("../fvp/plat.rs");

#[cfg(feature = "qemu")]
include!("../qemu/plat.rs");
