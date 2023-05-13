use chrono::Local;
use std::{
    collections::HashMap,
    fmt::Display,
    process::exit,
};

/// A tagging system that (1) indicates how users should interpret the log message
/// and (2) presents a linear scale for filtering output.
pub enum Level {
    /// Useful for debugging, not output by default
    Debug,
    /// Spam, depending on the beholder
    Info,
    /// Unexpected but non-disruptive issues, may or may not indicate a real issue
    Warn,
    /// Issues that may lead to undesired termination of a process, but not the entire
    /// program
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }.fmt(f)
    }
}

/// A comprehensive structural error type, intended exclusively for human
/// consumption.
#[derive(Debug, Clone)]
pub struct Error {
    /// What happened, what was expected to have happened, in what context this
    /// occurred (larger picture).
    message: &'static str,
    /// KV pairs with additional/dynamically generated information, specific context.
    /// By putting information here rather than in the message, you don't need to worry
    /// about delimination.
    attrs: HashMap<&'static str, String>,
    /// If an error occurs due to one or more other errors, those errors are here.
    causes: Vec<Error>,
    /// Errors that occur during error handling
    incidental: Vec<Error>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{:#?}", self).fmt(f)
    }
}

/// Use to convert other errors into logerr errors. Call
/// `err.logerr(log, "Doing XYZ")`.
pub trait ToLogError {
    fn logerr(self, log: &Log, message: &'static str) -> Error;
}

impl<T: std::error::Error> ToLogError for T {
    fn logerr(self, log: &Log, message: &'static str) -> Error {
        return log.err(message, |a| {
            a.insert("err", self.to_string());
        });
    }
}

/// Use to convert error results into logerr error results.  Call
/// `res.logerr(log, "Doing XYZ")?`.
pub trait ResultToLogError<T> {
    fn logerr(self, log: &Log, message: &'static str) -> Result<T, Error>;
}

impl<T, E: std::error::Error> ResultToLogError<T> for Result<T, E> {
    fn logerr(self, log: &Log, message: &'static str) -> Result<T, Error> {
        match self {
            Err(e) => Err(e.logerr(log, message)),
            Ok(x) => Ok(x),
        }
    }
}

/// Log a fatal error and terminate the program.
pub fn fatal(e: Error) -> ! {
    eprintln!("{} FATAL: Exiting due to error:\n{:#?}", Local::now().to_rfc3339(), e);
    exit(1)
}

/// Create an error and add context to one or more source errors (causes).
pub fn errors(message: &'static str, es: Vec<Error>) -> Error {
    return Error {
        message: message,
        attrs: HashMap::new(),
        causes: es,
        incidental: vec![],
    };
}

/// A logger, but more generally a store of current context (what's going on). The
/// context is used both for adding information to log messages and errors
/// generated during this context.
#[derive(Clone)]
pub struct Log {
    attrs: HashMap<&'static str, String>,
}

impl Log {
    pub fn new() -> Self {
        Self { attrs: HashMap::new() }
    }

    #[doc(hidden)]
    pub fn fork(&self, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Self {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        return Log { attrs: new_attrs };
    }

    #[doc(hidden)]
    pub fn print(&self, l: Level, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);

        #[derive(Debug)]
        struct Event<'a> {
            #[allow(dead_code)]
            message: &'a str,
            #[allow(dead_code)]
            attrs: &'a HashMap<&'static str, String>,
        }

        eprintln!("{} {}: {:#?}", Local::now().to_rfc3339(), l, Event {
            message: message,
            attrs: &self.attrs,
        })
    }

    #[doc(hidden)]
    pub fn err(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        Error {
            message: message,
            attrs: new_attrs,
            causes: vec![],
            incidental: vec![],
        }
    }

    #[doc(hidden)]
    pub fn also(
        &self,
        e: Error,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        let mut new_incidental = e.incidental.clone();
        new_incidental.push(Error {
            message: message,
            attrs: new_attrs,
            causes: vec![],
            incidental: vec![],
        });
        Error {
            message: e.message,
            attrs: e.attrs,
            causes: e.causes,
            incidental: new_incidental,
        }
    }
}

/// Create a new `Log` that extends the source context.  Use like
/// `let new_log = fork!(log, newkey=newvalue, ...);`. Values must have the method
/// `to_string()`.
#[macro_export]
macro_rules! fork{
    ($l: expr, $($k: ident = $v: expr), *) => {
        $l.fork(| attrs | {
            $(attrs.insert(stringify!($k), $v.to_string());) *
        })
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! log{
    ($l: expr, $lv: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $l.print($lv, $m, | attrs | {
            $(attrs.insert(stringify!($k), $v.to_string());) *
        })
    };
}

/// Log at the debug level.  Use like `log_debug!(log, key=value, ...);`. Values
/// must have the method `to_string()`.
#[macro_export]
macro_rules! debug{
    ($l: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $crate:: log !($l, $crate:: Level:: Debug, $m $(, $k = $v) *)
    };
}

/// Log at the info level.  Use like `log_info!(log, key=value, ...);`. Values must
/// have the method `to_string()`.
#[macro_export]
macro_rules! log_info{
    ($l: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $crate:: log !($l, $crate:: Level:: Info, $m $(, $k = $v) *)
    };
}

/// Log at the warn level.  Use like `log_warn!(log, key=value, ...);`. Values must
/// have the method `to_string()`.
#[macro_export]
macro_rules! log_warn{
    ($l: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $crate:: log !($l, $crate:: Level:: Warn, $m $(, $k = $v) *)
    };
}

/// Log at the error level.  Use like `log_err!(log, key=value, ...);`. Values must
/// have the method `to_string()`.
#[macro_export]
macro_rules! log_err{
    ($l: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $crate:: log !($l, $crate:: Level:: Error, $m $(, $k = $v) *)
    };
}

/// Create an error from the log context.  Use like
/// `err!(log, "Stuff broke", key=value, ...);`. Values must have the method
/// `to_string()`.
#[macro_export]
macro_rules! err{
    ($l: expr, $m: literal $(, $k: ident = $v: expr) *) => {
        $l.err($m, | attrs | {
            $(attrs.insert(stringify!($k), $v.to_string());) *
        })
    };
}

/// Extend the base error with a new incidental (occurred while handling the base
/// error) error.  Use like
/// `also!(log, base_e, "Stuff broke while other things broke", key=value, ...);`.
/// Values must have the method `to_string()`.
#[macro_export]
macro_rules! also{
    ($l: expr, $e: expr, $m: literal $(, $k: ident = $v: expr), *) => {
        $l.also($e, $m, | attrs | {
            $(attrs.insert(stringify!($k), $v.to_string());) *
        })
    };
}
