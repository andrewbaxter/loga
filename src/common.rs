use std::hash::Hash;

pub struct FlagStyle {
    /// Color to render the log message body;
    pub body_style: console::Style,
    /// Color to render the flag label;
    pub label_style: console::Style,
    /// Text for the log message flag label (ex: WARN, DEBUG, etc)
    pub label: &'static str,
}

/// This trait defines the flags value used for gating logging messages.  You
/// should define a new bitflags type and then implement this on it.  See
/// `StandardFlag` for an example.
pub trait Flag: Hash + Eq + Clone {
    fn style(self) -> FlagStyle;
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
