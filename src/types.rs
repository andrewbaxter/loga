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
use crate::{
    DebugDisplay,
    ea,
    Flags,
};

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
    pub fn context_with(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
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
    pub fn log_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Error {
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
    pub fn log_context_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
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
        #[cfg(debug_assertions)]
        return Error::from(value.dbg_str());
        #[cfg(not(debug_assertions))]
        return Error::from(value);
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
pub struct Log<F: Flags> {
    pub(crate) attrs: HashMap<&'static str, String>,
    pub(crate) flags: Option<F>,
}

impl<F: Flags> Default for Log<F> {
    fn default() -> Self {
        Self {
            attrs: HashMap::new(),
            flags: None,
        }
    }
}

impl<F: Flags> Log<F> {
    /// Create a new logger (defaults to Debug level, change with `with_level`). You
    /// may want to alias this with your flag type of choice.
    pub fn new() -> Self {
        return Log {
            attrs: HashMap::new(),
            flags: None,
        };
    }

    /// Create a new `Log` that inherits attributes from the base logging context.  Use
    /// like `let new_log = log.fork(ea!(newkey = newvalue, ...));`.
    pub fn fork(&self, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Self {
        let mut new_attrs = self.attrs.clone();
        attrs(&mut new_attrs);
        return Log {
            attrs: new_attrs,
            flags: self.flags,
        };
    }

    /// Initialize or replace flags and return a new Log instance.
    pub fn with_flags(self, flags: F) -> Self {
        return Log {
            attrs: self.attrs,
            flags: Some(flags),
        };
    }

    /// Log a message.  The message will only be rendered and output if any of the
    /// specified flags are set.
    pub fn log(&self, flags: F, message: &'static str) {
        self.log_with(flags, message, ea!());
    }

    /// Log a message.  The attributes will only be evaluated and the message will only
    /// be rendered and output if any of the specified flags are set.
    pub fn log_with(&self, flags: F, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) {
        let mask = self.flags.expect("Can't log because flags aren't set in this logger!");
        if !mask.intersects(flags) {
            return;
        }
        self.log_err(flags, self.err_with(message, attrs));
    }

    pub fn log_err(&self, flags: F, e: Error) {
        let mask = self.flags.expect("Can't log because flags aren't set in this logger!");
        if !mask.intersects(flags) {
            return;
        }
        let style = flags.style();
        let (head, body) = e.render();
        log(style.body_style, style.label_style, style.label, head, body);
    }

    /// Create a new error including the attributes in this logging context.
    pub fn err(&self, message: &'static str) -> Error {
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
    pub fn err_with(&self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
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
