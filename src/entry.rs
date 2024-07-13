use console::Style as TextStyle;
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    io::Write,
    process::exit,
};
use crate::types::{
    log,
    Error,
    Error_,
};

/// Create a new error. If you want to inherit attributes from a logging context,
/// see `Log::err`.
pub fn err(message: impl ToString) -> Error {
    return Error(Box::new(Error_ {
        message: message.to_string(),
        attrs: HashMap::new(),
        context: vec![],
        causes: vec![],
        incidental: vec![],
    }));
}

/// Create a new error and attach attributes. If you want to inherit attributes
/// from a logging context, see `Log::err`.
pub fn err_with(message: impl ToString, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
    let mut new_attrs = HashMap::new();
    attrs(&mut new_attrs);
    return Error(Box::new(Error_ {
        message: message.to_string(),
        attrs: new_attrs,
        context: vec![],
        causes: vec![],
        incidental: vec![],
    }));
}

/// Create an error from multiple errors
pub fn agg_err(message: impl ToString, errs: Vec<Error>) -> Error {
    return Error(Box::new(Error_ {
        message: message.to_string(),
        attrs: HashMap::new(),
        context: vec![],
        causes: errs,
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
        message: message.to_string(),
        attrs: new_attrs,
        context: vec![],
        causes: errs,
        incidental: vec![],
    }));
}

/// Log a fatal error and terminate the program.
pub fn fatal(e: Error) -> ! {
    let body_color = TextStyle::new().for_stderr().red();
    let level_color = TextStyle::new().for_stderr().red().bold();
    let mut node = e.build_render_nodes(&HashSet::new());
    node.title = format!("Exiting due to error: {}", node.title).into();
    log(body_color, level_color, "FATAL", node);
    _ = std::io::stderr().flush();
    exit(1)
}
