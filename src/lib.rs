pub mod entry;
pub mod common;
pub mod types;
pub mod conversion;

pub use types::{
    Error,
    Log,
    Level,
};
pub use entry::{
    new,
    new_err,
    new_err_with,
    new_agg_err,
    new_agg_err_with,
    fatal,
};
pub use common::DebugDisplay;
pub use conversion::{
    ErrContext,
    ResultContext,
};
