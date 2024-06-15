#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Level(pub(crate) i8);

pub const NONE: Level = Level(0);
pub const DEBUG: Level = Level(1);
pub const INFO: Level = Level(2);
pub const WARN: Level = Level(3);
pub const ERR: Level = Level(4);

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
