use std::{
    any::Any,
    sync::OnceLock,
};
use console::Style as TextStyle;

pub struct FlagsStyle {
    /// Color to render the log message body;
    pub body_style: TextStyle,
    /// Color to render the flag label;
    pub label_style: TextStyle,
    /// Text for the log message flag label (ex: WARN, DEBUG, etc)
    pub label: &'static str,
}

/// This trait defines the flags value used for gating logging messages.  You
/// should define a new bitflags type and then implement this on it.  See
/// `StandardFlags` for an example.
pub trait Flags: bitflags::Flags + Send + Sync + Copy + Eq {
    fn style(self) -> FlagsStyle;
}

pub(crate) fn flags_<T: Flags>(init: T) -> T {
    static FLAGS: OnceLock<Box<dyn Any + Sync + Send>> = OnceLock::new();
    let res =
        *FLAGS
            .get_or_init(move || Box::new(init))
            .downcast_ref::<T>()
            .expect("Flags of another type have already been initialized!");
    if res != init {
        panic!("Flags have already been initialized with different values!");
    }
    return res;
}

/// Turn key/values into a lambda for extending attributes, used in various log and
/// error functions.
#[macro_export]
macro_rules! ea{
    ($($k: ident = $v: expr), *) => {
        | _attrs | {
            $(_attrs.insert(stringify!($k), $v.to_string());) *
        }
    };
}

/// A helper to easily generate debug strings for types implementing `Debug`.
pub trait DebugDisplay {
    fn dbg_str(&self) -> String;
    fn pretty_dbg_str(&self) -> String;
}

impl<T: std::fmt::Debug> DebugDisplay for T {
    fn dbg_str(&self) -> String {
        return format!("{:?}", self);
    }

    fn pretty_dbg_str(&self) -> String {
        return format!("{:#?}", self);
    }
}
