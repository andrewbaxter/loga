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
    StandardFlags,
    StandardLog,
};
pub use common::{
    DebugDisplay,
    Flags,
    FlagsStyle,
};
pub use conversion::{
    ErrContext,
    ResultContext,
};

/// Re-exported dependencies used in interfaces, etc.
pub mod republish {
    pub use console;
    pub use bitflags;
}
