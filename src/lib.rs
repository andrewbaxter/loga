use chrono::Local;
use console::{
    Style,
    measure_text_width,
};
use textwrap::{
    wrap,
    Options,
};
use std::{
    collections::HashMap,
    fmt::{
        Display,
    },
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
        | _attrs | {
            $(_attrs.insert(stringify!($k), $v.to_string());) *
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

enum RenderNode {
    Leaf(String),
    KVLeaf(String, String),
    Branch(Vec<RenderNode>),
}

/// Create a new error. If you want to inherit attributes from a logging context,
/// see `Log::new_err`.
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
/// from a logging context, see `Log::new_err`.
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

impl Error {
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

    fn render(&self) -> (String, RenderNode) {
        let message;
        let mut children = vec![];
        let highlight_style = Style::new().for_stderr().blue();
        let dark_style = Style::new().for_stderr().dim();
        match &self.0.inner {
            Error_2::Simple(e) => {
                message = e.clone();
            },
            Error_2::Full(e) => {
                message = e.message.to_string();
                for a in &e.attrs {
                    children.push(
                        RenderNode::KVLeaf(
                            dark_style.apply_to(format!("{} = ", a.0)).to_string(),
                            dark_style.apply_to(a.1.trim()).to_string(),
                        ),
                    )
                }
                if e.causes.len() > 0 {
                    children.push(RenderNode::Leaf(highlight_style.apply_to("Caused by:").to_string()));
                    for e in &e.causes {
                        let (head, body) = e.render();
                        children.push(RenderNode::Branch(vec![RenderNode::KVLeaf("- ".to_string(), head), body]));
                    }
                }
            },
        }
        if self.0.incidental.len() > 0 {
            children.push(RenderNode::Leaf("Incidentally:".to_string()));
            for e in &self.0.incidental {
                let (head, body) = e.render();
                children.push(RenderNode::Branch(vec![RenderNode::KVLeaf("- ".to_string(), head), body]));
            }
        }
        return (message, RenderNode::Branch(children));
    }
}

fn render(root: RenderNode) -> String {
    let mut out = String::new();
    let mut stack = vec![(0usize, &root)];
    while let Some((indent_count, top)) = stack.pop() {
        let indent = "  ".repeat(indent_count);
        match top {
            RenderNode::Leaf(l) => {
                for line in wrap(&l, Options::with_termwidth().initial_indent(&indent).subsequent_indent(&indent)) {
                    out.push_str(&line);
                    out.push('\n');
                }
            },
            RenderNode::KVLeaf(k, v) => {
                for line in wrap(
                    &v,
                    Options::with_termwidth()
                        .initial_indent(&format!("{}{}", indent, k))
                        .subsequent_indent(&format!("{}{}", indent, " ".repeat(measure_text_width(k)))),
                ) {
                    out.push_str(&line);
                    out.push('\n');
                }
            },
            RenderNode::Branch(b) => {
                stack.extend(b.iter().rev().map(|e| (indent_count + 1, e)));
            },
        }
    }
    return out;
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

fn log(body_color: Style, level_color: Style, level_text: &str, head: String, body: RenderNode) {
    eprint!(
        "{}{}",
        render(
            RenderNode::KVLeaf(
                format!("{} {}: ", body_color.apply_to(Local::now().to_rfc3339()), level_color.apply_to(level_text)),
                head,
            ),
        ),
        render(body)
    );
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

/// A logger, but more generally a store of current context (what's going on). The
/// context is used both for adding information to log messages and errors
/// generated during this context.
#[derive(Clone)]
pub struct Log {
    filter_priority: i8,
    attrs: HashMap<&'static str, String>,
}

/// Create a new logger.
pub fn new(level: Level) -> Log {
    Log {
        filter_priority: level.priority(),
        attrs: HashMap::new(),
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            filter_priority: Level::Info.priority(),
            attrs: HashMap::new(),
        }
    }
}

impl Log {
    /// Create a new `Log` that inherits attributes from the base logging context.  Use
    /// like `let new_log = log.fork(ea!(newkey = newvalue, ...));`.
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
        let (body_color, level_color) = match l {
            Level::Debug => (Style::new().for_stderr().black().bright(), Style::new().for_stderr().black().bright()),
            Level::Info => (Style::new().for_stderr().black(), Style::new().for_stderr().black()),
            Level::Warn => (Style::new().for_stderr().black(), Style::new().for_stderr().yellow()),
            Level::Error => (Style::new().for_stderr().black(), Style::new().for_stderr().red()),
        };
        let (head, body) = self.new_err_with(message, attrs).render();
        log(body_color, level_color, &l.to_string(), head, body);
    }

    /// Log a message at the debug level.
    pub fn debug(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Debug, message, attrs);
    }

    /// Log a message at the info level.
    pub fn info(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Info, message, attrs);
    }

    /// Log a message at the warn level.
    pub fn warn(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log(Level::Warn, message, attrs);
    }

    /// Log a message at the error level.
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

    /// Log an error with an additional message at the debug level.
    pub fn debug_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Debug, e, message, attrs);
    }

    /// Log an error with an additional message at the info level.
    pub fn info_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Info, e, message, attrs);
    }

    /// Log an error with an additional message at the warn level.
    pub fn warn_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Warn, e, message, attrs);
    }

    /// Log an error with an additional message at the error level.
    pub fn err_e(&self, e: Error, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        self.log_e(Level::Error, e, message, attrs);
    }

    /// Create a new error including the attributes in this logging context.
    pub fn new_err(&self, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                causes: vec![],
                attrs: HashMap::new(),
            }),
            incidental: vec![],
        }));
    }

    /// Create a new error including the attributes in this logging context and merging
    /// additional attributes.
    pub fn new_err_with(
        &self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
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
    /// Add a simple context string onto an error, converting it to `loga::Error` in
    /// the process.
    fn context(self, message: &'static str) -> Error;

    /// Add a simple context string and attributes pairs onto an error, converting it
    /// to `loga::Error` in the process.
    fn context_with(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error;

    /// Add attributes from the log as well as a simple context string to an error,
    /// converting it to `loga::Error` in the process.
    fn log_context(self, log: &Log, message: &'static str) -> Error;

    /// Add attributes from the log as well as the specified attributes and a simple
    /// context string to an error, converting it to `loga::Error` in the process.
    fn log_context_with(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error;
}

impl<T: Into<Error>> ErrContext for T {
    fn context(self, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: HashMap::new(),
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }

    fn context_with(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
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

    fn log_context(self, log: &Log, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: log.attrs.clone(),
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }

    fn log_context_with(
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
    /// If the value is Err/None, add a simple context string onto an error, converting
    /// it to `loga::Error` in the process.
    fn context(self, message: &'static str) -> Result<O, Error>;

    /// If the value is Err/None, add a simple context string and attributes pairs onto
    /// an error, converting it to `loga::Error` in the process.
    fn context_with(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;

    /// If the value is Err/None, add attributes from the log as well as a simple
    /// context string to an error, converting it to `loga::Error` in the process.
    fn log_context(self, log: &Log, message: &'static str) -> Result<O, Error>;

    /// If the value is Err/None, add attributes from the log as well as the specified
    /// attributes and a simple context string to an error, converting it to
    /// `loga::Error` in the process.
    fn log_context_with(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;

    // If the value is Err/None, consume it, logging it with the additional context
    // message.
    fn warn(self, log: &Log, message: &'static str);

    // If the value is Err/None, consume it, logging it with the additional context
    // message and attributes.
    fn warn_with(self, log: &Log, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ());
}

impl<O, T: Into<Error>> ResultContext<O> for Result<O, T> {
    fn context(self, message: &'static str) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.context(message)),
        }
    }

    fn context_with(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.context_with(message, attrs)),
        }
    }

    fn log_context(self, log: &Log, message: &'static str) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.log_context(log, message)),
        }
    }

    fn log_context_with(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.log_context_with(log, message, attrs)),
        }
    }

    fn warn(self, log: &Log, message: &'static str) {
        if let Err(e) = self {
            log.warn_e(e.into(), message, ea!());
        }
    }

    fn warn_with(self, log: &Log, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        if let Err(e) = self {
            log.warn_e(e.into(), message, attrs);
        }
    }
}

impl<O> ResultContext<O> for Option<O> {
    fn context(self, message: &'static str) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(err(message)),
        }
    }

    fn context_with(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(err_with(message, attrs)),
        }
    }

    fn log_context(self, log: &Log, message: &'static str) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.new_err(message)),
        }
    }

    fn log_context_with(
        self,
        log: &Log,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.new_err_with(message, attrs)),
        }
    }

    fn warn(self, log: &Log, message: &'static str) {
        if self.is_none() {
            log.warn(message, ea!());
        }
    }

    fn warn_with(self, log: &Log, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        if self.is_none() {
            log.warn(message, attrs);
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
