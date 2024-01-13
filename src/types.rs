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

#[derive(Debug, Clone)]
pub struct FullError {
    pub message: &'static str,
    pub attrs: HashMap<&'static str, String>,
    pub causes: Vec<Error>,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Error_2 {
    Simple(String),
    Full(FullError),
}

#[derive(Debug, Clone)]
pub(crate) struct Error_ {
    pub(crate) inner: Error_2,
    /// Errors that occur during error handling
    pub(crate) incidental: Vec<Error>,
}

/// A comprehensive structural error type, intended exclusively for human
/// consumption.
#[derive(Debug, Clone)]
pub struct Error(pub(crate) Box<Error_>);

pub(crate) enum RenderNode {
    Leaf(String),
    KVLeaf(String, String),
    Branch(Vec<RenderNode>),
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

    pub(crate) fn render(&self) -> (String, RenderNode) {
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

    /// Return a new error adding a simple string message as a layer of context.
    pub fn context(self, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: HashMap::new(),
                causes: vec![self],
            }),
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context with a message and attributes.
    fn context_with(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: new_attrs,
                causes: vec![self],
            }),
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context including all attributes in the
    /// provided log along with the specified message.
    pub fn log_context(self, log: &Log, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: log.attrs.clone(),
                causes: vec![self],
            }),
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context including all attributes in the
    /// provided log along with the specified message and new attributes.
    pub fn log_context_with(
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
                causes: vec![self],
            }),
            incidental: vec![],
        }));
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

pub(crate) fn log(body_color: Style, level_color: Style, level_text: &str, head: String, body: RenderNode) {
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

/// A store of context with methods for logging and creating errors expressing that
/// context.
///
/// Unnecessary detail: This is called "Log" because this object will take the
/// place of a log or logger in most code and thus is unlikely to overlap.  Naming
/// it "Context" or similar would possibly conflict with other domains' "context"
/// objects. The two hardest things when programming, as they say - no need to make
/// things harder.
#[derive(Clone)]
pub struct Log {
    pub(crate) filter_priority: i8,
    pub(crate) attrs: HashMap<&'static str, String>,
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
    /// Set the log level.  If you don't call this the default level is Debug.  Log
    /// level only applies when logging, not when generating errors or forking.
    pub fn with_level(mut self, level: Level) -> Self {
        self.filter_priority = level.priority();
        return self;
    }

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
        let (body_color, level_color) = match l {
            Level::Debug => (Style::new().for_stderr().black().bright(), Style::new().for_stderr().black().bright()),
            Level::Info => (Style::new().for_stderr().black(), Style::new().for_stderr().black()),
            Level::Warn => (Style::new().for_stderr().black(), Style::new().for_stderr().yellow()),
            Level::Error => (Style::new().for_stderr().black(), Style::new().for_stderr().red()),
        };
        let (head, body) = e.context_with(message, attrs).render();
        log(body_color, level_color, &l.to_string(), head, body);
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
