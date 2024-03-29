use console::Style as TextStyle;
use std::{
    collections::HashMap,
    process::exit,
    io::Write,
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
        Flag,
    },
    FlagStyle,
};

/// Create a new error. If you want to inherit attributes from a logging context,
/// see `Log::err`.
pub fn err(message: impl ToString) -> Error {
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message.to_string(),
            attrs: HashMap::new(),
            causes: vec![],
        }),
        incidental: vec![],
    }));
}

/// Create a new error and attach attributes. If you want to inherit attributes
/// from a logging context, see `Log::err`.
pub fn err_with(message: impl ToString, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
    let mut new_attrs = HashMap::new();
    attrs(&mut new_attrs);
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message.to_string(),
            attrs: new_attrs,
            causes: vec![],
        }),
        incidental: vec![],
    }));
}

/// Create an error from multiple errors
pub fn agg_err(message: impl ToString, errs: Vec<Error>) -> Error {
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message.to_string(),
            attrs: HashMap::new(),
            causes: errs,
        }),
        incidental: vec![],
    }));
}

/// Create an error from multiple errors, attaching attributes
pub fn agg_err_with(
    message: impl ToString,
    errs: Vec<Error>,
    attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
) -> Error {
    let mut new_attrs = HashMap::new();
    attrs(&mut new_attrs);
    return Error(Box::new(Error_ {
        inner: Error_2::Full(FullError {
            message: message.to_string(),
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
    _ = std::io::stderr().flush();
    exit(1)
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum StandardFlag {
    Error,
    Warning,
    Info,
    Debug,
}

impl Flag for StandardFlag {
    fn style(self) -> FlagStyle {
        match self {
            StandardFlag::Debug => FlagStyle {
                body_style: TextStyle::new().for_stderr().black().bright(),
                label_style: TextStyle::new().for_stderr().black().bright(),
                label: "DEBUG",
            },
            StandardFlag::Info => FlagStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().black(),
                label: "INFO",
            },
            StandardFlag::Warning => FlagStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().yellow(),
                label: "WARN",
            },
            StandardFlag::Error => FlagStyle {
                body_style: TextStyle::new().for_stderr().black(),
                label_style: TextStyle::new().for_stderr().red(),
                label: "ERROR",
            },
        }
    }
}

/// A logger using a preconfigured flag set.
pub type StandardLog = Log<StandardFlag>;
