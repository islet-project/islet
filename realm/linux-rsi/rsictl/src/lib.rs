mod ioctl;
mod token;
// the below can be removed at some point, debug purposes only
mod token_c;


pub use ioctl::kernel::MAX_MEASUR_LEN;
pub use ioctl::kernel::CHALLENGE_LEN;
pub use ioctl::kernel::MAX_TOKEN_LEN;

pub use ioctl::abi_version;
pub use ioctl::attestation_token;
pub use ioctl::measurement_extend;
pub use ioctl::measurement_read;

pub use token::TokenError;
pub use token::verifier::verify_token;
pub use token::dumper::print_token;

pub use token_c::TokenError as CTokenError;
pub use token_c::verify_token as c_verify_token;
pub use token_c::print_raw_token as c_print_raw_token;
pub use token_c::print_token as c_print_token;
pub use token_c::print_token_rust as c_print_token_rust;
