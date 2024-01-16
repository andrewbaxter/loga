use std::{
    any::Any,
    sync::OnceLock,
};
use bitflags::bitflags;
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

bitflags!{
    #[derive(PartialEq, Eq, Clone, Copy)] pub struct StandardFlags: u8 {
        const DEBUG = 1 << 0;
        const INFO = 1 << 1;
        const WARN = 1 << 2;
        const ERROR = 1 << 3;
        const FATAL = 1 << 4;
    }
}

impl Flags for StandardFlags {
    fn style(self) -> FlagsStyle {
        match self.iter().next().unwrap() {
            StandardFlags::DEBUG => FlagsStyle {
                body_style: TextStyle::new().for_stderr().black().bright(),
                label_style: TextStyle::new().for_stderr().black().bright(),
                label: "DEBUG",
            },
            StandardFlags::INFO => FlagsStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().black(),
                label: "INFO",
            },
            StandardFlags::WARN => FlagsStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().yellow(),
                label: "WARN",
            },
            StandardFlags::ERROR => FlagsStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().red(),
                label: "ERROR",
            },
            StandardFlags::FATAL => FlagsStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().black(),
                label: "FATAL",
            },
            _ => panic!(),
        }
    }
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
