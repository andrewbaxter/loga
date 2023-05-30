use chrono::Local;
use console::Style;
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

impl Level {
    pub fn priority(&self) -> i8 {
        match self {
            Level::Debug => 10,
            Level::Info => 20,
            Level::Warn => 30,
            Level::Error => 40,
        }
    }
}

/// Turn key/values into a lambda for extending attributes, used in various log and
/// error functions.
#[macro_export]
macro_rules! ea{
    ($($k: ident = $v: expr), *) => {
        | attrs | {
            $(attrs.insert(stringify!($k), $v.to_string());) *
        }
    };
}

#[derive(Debug, Clone)]
pub struct FullError {
    pub message: &'static str,
    pub attrs: HashMap<&'static str, String>,
    pub causes: Vec<Error>,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Error_2 {
    Simple(String),
    Full(FullError),
}

#[derive(Debug, Clone)]
pub struct Error_ {
    pub inner: Error_2,
    /// Errors that occur during error handling
    pub incidental: Vec<Error>,
}

/// A comprehensive structural error type, intended exclusively for human
/// consumption.
#[derive(Debug, Clone)]
pub struct Error(Box<Error_>);

impl Error {
    pub fn new(message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
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

    pub fn from(x: impl Display) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Simple(x.to_string()),
            incidental: vec![],
        }));
    }

    /// Extend the base error with a new incidental (occurred while handling the base
    /// error) error.  Use like `e.also(log, new_e);`.
    pub fn also(mut self, incidental: Error) -> Error {
        self.0.incidental.push(incidental);
        return self;
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = String::new();
        match &self.0.inner {
            Error_2::Simple(e) => {
                text.push_str(&e);
            },
            Error_2::Full(e) => {
                text.push_str(e.message);
                for (k, v) in &e.attrs {
                    text.push_str(", ");
                    text.push_str(k);
                    text.push_str("=");
                    text.push_str(v);
                }
                if !e.causes.is_empty() {
                    text.push_str(", causes [");
                    for i in &e.causes {
                        text.push_str(&i.to_string());
                        text.push_str(" ");
                    }
                    text.push_str("]");
                }
            },
        }
        if !self.0.incidental.is_empty() {
            text.push_str(", incidental [");
            for i in &self.0.incidental {
                text.push_str(&i.to_string());
                text.push_str(" ");
            }
            text.push_str("]");
        }
        text.fmt(f)
    }
}

impl<T: std::error::Error> From<T> for Error {
    fn from(value: T) -> Self {
        Error::from(value)
    }
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

/// Log a fatal error and terminate the program.
pub fn fatal(e: Error) -> ! {
    let body_color = Style::new().for_stderr().magenta();
    let level_color = Style::new().for_stderr().magenta().bold();
    eprintln!(
        "{} {}: {}",
        body_color.apply_to(Local::now().to_rfc3339()),
        level_color.apply_to("FATAL"),
        body_color.apply_to(format!("Exiting due to error:\n{:#?}", e))
    );
    exit(1)
}

/// A logger, but more generally a store of current context (what's going on). The
/// context is used both for adding information to log messages and errors
/// generated during this context.
#[derive(Clone)]
pub struct Log {
    filter_priority: i8,
    attrs: HashMap<&'static str, String>,
}

impl Log {
    pub fn new(level: Level) -> Self {
        Self {
            filter_priority: level.priority(),
            attrs: HashMap::new(),
        }
    }

    /// Create a new `Log` that extends the source context.  Use like
    /// `let new_log = log.fork(ea!(newkey = newvalue, ...));`. Values must have the
    /// method `to_string()`.
    pub fn fork(&self, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Self {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        return Log {
            filter_priority: self.filter_priority,
            attrs: new_attrs,
        };
    }

    pub fn log(&self, l: Level, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        if l.priority() < self.filter_priority {
            return;
        }
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);

        #[derive(Debug)]
        struct Event<'a> {
            #[allow(dead_code)]
            message: &'a str,
            #[allow(dead_code)]
            attrs: &'a HashMap<&'static str, String>,
        }

        let (body_color, level_color) = match l {
            Level::Debug => (Style::new().for_stderr().black().bright(), Style::new().for_stderr().black().bright()),
            Level::Info => (Style::new().for_stderr().black(), Style::new().for_stderr().black()),
            Level::Warn => (Style::new().for_stderr().black(), Style::new().for_stderr().red()),
            Level::Error => (Style::new().for_stderr().black(), Style::new().for_stderr().red().bold()),
        };
        eprintln!(
            "{} {}: {}",
            body_color.apply_to(Local::now().to_rfc3339()),
            level_color.apply_to(l),
            body_color.apply_to(Event {
                message: message,
                attrs: &new_attrs,
            }.pretty_dbg_str())
        )
    }

    pub fn debug(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Debug, message, attrs);
    }

    pub fn info(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Info, message, attrs);
    }

    pub fn warn(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Warn, message, attrs);
    }

    pub fn err(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Error, message, attrs);
    }

    pub fn log_e(
        &self,
        l: Level,
        e: Error,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        if l.priority() < self.filter_priority {
            return;
        }
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);

        #[derive(Debug)]
        struct Event<'a> {
            #[allow(dead_code)]
            message: &'a str,
            #[allow(dead_code)]
            attrs: &'a HashMap<&'static str, String>,
            #[allow(dead_code)]
            error: Error,
        }

        let (body_color, level_color) = match l {
            Level::Debug => (Style::new().for_stderr().black().bright(), Style::new().for_stderr().black().bright()),
            Level::Info => (Style::new().for_stderr().black(), Style::new().for_stderr().black()),
            Level::Warn => (Style::new().for_stderr().black(), Style::new().for_stderr().red()),
            Level::Error => (Style::new().for_stderr().black(), Style::new().for_stderr().red().bold()),
        };
        eprintln!(
            "{} {}: {}",
            body_color.apply_to(Local::now().to_rfc3339()),
            level_color.apply_to(l),
            body_color.apply_to(Event {
                message: message,
                attrs: &new_attrs,
                error: e,
            }.pretty_dbg_str())
        )
    }

    pub fn debug_e(
        &self,
        e: Error,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        self.log_e(Level::Debug, e, message, attrs);
    }

    pub fn info_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Info, e, message, attrs);
    }

    pub fn warn_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Warn, e, message, attrs);
    }

    pub fn err_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Error, e, message, attrs);
    }

    pub fn new_err(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                causes: vec![],
                attrs: new_attrs,
            }),
            incidental: vec![],
        }));
    }
}

pub trait ErrContext {
    fn context(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error;
    fn log_context(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error;
}

impl<T: Into<Error>> ErrContext for T {
    fn context(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: new_attrs,
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }

    fn log_context(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
        let mut new_attrs = log.attrs.clone();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: new_attrs,
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }
}

pub trait ResultContext<O> {
    fn context(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;
    fn log_context(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;
}

impl<O, T: Into<Error>> ResultContext<O> for Result<O, T> {
    fn context(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.context(message, attrs)),
        }
    }

    fn log_context(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.log_context(log, message, attrs)),
        }
    }
}

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
