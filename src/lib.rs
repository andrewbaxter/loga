pub mod entry;
pub mod common;
pub mod types;
pub mod conversion;

pub use types::{
    Error,
    Log,
};
pub use entry::{
    err,
    err_with,
    agg_err,
    agg_err_with,
    fatal,
    StandardFlag,
    StandardLog,
};
pub use common::{
    DebugDisplay,
    Flag,
    FlagStyle,
};
pub use conversion::{
    ErrContext,
    ResultContext,
};

/// Re-exported dependencies used in interfaces, etc.
pub mod republish {
    pub use console;
}
