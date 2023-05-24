use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(target_arch="x86_64")] {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct Report {
            pub buffer: Vec<u8>,
            pub user_data: Vec<u8>,
        }
    } else {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct Report {
            pub buffer: Vec<u8>,
        }
    }
}
