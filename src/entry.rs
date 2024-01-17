use console::Style as TextStyle;
use std::{
    collections::HashMap,
    process::exit,
};
use crate::{
    types::{
        Error,
        Error_2,
        FullError,
        Error_,
        log,
        Log,
    },
    common::{
        Flags,
    },
    FlagsStyle,
};

/// Create a new error. If you want to inherit attributes from a logging context,
/// see `Log::err`.
pub fn err(message: &'static str) -> Error {
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message,
            attrs: HashMap::new(),
            causes: vec![],
        }),
        incidental: vec![],
    }));
}

/// Create a new error and attach attributes. If you want to inherit attributes
/// from a logging context, see `Log::err`.
pub fn err_with(message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
    let mut new_attrs = HashMap::new();
    attrs(&mut new_attrs);
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message,
            attrs: new_attrs,
            causes: vec![],
        }),
        incidental: vec![],
    }));
}

/// Create an error from multiple errors
pub fn agg_err(message: &'static str, errs: Vec<Error>) -> Error {
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message,
            attrs: HashMap::new(),
            causes: errs,
        }),
        incidental: vec![],
    }));
}

/// Create an error from multiple errors, attaching attributes
pub fn agg_err_with(
    message: &'static str,
    errs: Vec<Error>,
    attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
) -> Error {
    let mut new_attrs = HashMap::new();
    attrs(&mut new_attrs);
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message,
            attrs: new_attrs,
            causes: errs,
        }),
        incidental: vec![],
    }));
}

/// Log a fatal error and terminate the program.
pub fn fatal(e: Error) -> ! {
    let body_color = TextStyle::new().for_stderr().red();
    let level_color = TextStyle::new().for_stderr().red().bold();
    let (head, body) = e.render();
    let head = level_color.apply_to(head).to_string();
    let foot = level_color.apply_to("Exited due to above error").to_string();
    log(body_color, level_color, "FATAL", head, body);
    eprintln!("{}", foot);
    exit(1)
}

bitflags::bitflags!{
    /// A basic set of flags if you don't want to define your own yet.
    #[derive(PartialEq, Eq, Clone, Copy)] pub struct StandardFlags: u8 {
        const FATAL = 1 << 0;
        const ERROR = 1 << 1;
        const WARN = 1 << 2;
        const INFO = 1 << 3;
        const DEBUG = 1 << 4;
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

/// A logger using a preconfigured flag set.
pub type StandardLog = Log<StandardFlags>;
