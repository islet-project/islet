#[cfg(feature = "gst_page_table")]
pub mod page_table;
#[cfg(feature = "gst_page_table")]
pub use page_table::*;

#[cfg(not(feature = "gst_page_table"))]
pub mod array;
#[cfg(not(feature = "gst_page_table"))]
pub use array::*;
