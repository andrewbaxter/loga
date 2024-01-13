use console::Style;
use std::{
    collections::HashMap,
    process::exit,
};
use crate::types::{
    Error,
    Error_2,
    FullError,
    Error_,
    log,
    Log,
    Level,
};

/// Create a new error. If you want to inherit attributes from a logging context,
/// see `Log::new_err`.
pub fn new_err(message: &'static str) -> Error {
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
/// from a logging context, see `Log::new_err`.
pub fn new_err_with(message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
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
pub fn new_agg_err(message: &'static str, errs: Vec<Error>) -> Error {
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
pub fn new_agg_err_with(
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
    let body_color = Style::new().for_stderr().red();
    let level_color = Style::new().for_stderr().red().bold();
    let (head, body) = e.render();
    let head = level_color.apply_to(head).to_string();
    let foot = level_color.apply_to("Exited due to above error").to_string();
    log(body_color, level_color, "FATAL", head, body);
    eprintln!("{}", foot);
    exit(1)
}

/// Create a new logger (defaults to Debug level, change with `with_level`).
pub fn new() -> Log {
    Log {
        filter_priority: Level::Debug.priority(),
        attrs: HashMap::new(),
    }
}
