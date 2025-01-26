use {
    crate::{
        ea,
        DebugDisplay,
        Level,
    },
    chrono::Local,
    console::{
        measure_text_width,
        Style,
    },
    std::{
        borrow::Cow,
        collections::{
            HashMap,
            HashSet,
        },
        fmt::Display,
        sync::Arc,
    },
    textwrap::{
        wrap,
        Options,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct Error_ {
    pub message: String,
    pub attrs: HashMap<&'static str, String>,
    pub context: Vec<Log>,
    pub causes: Vec<Error>,
    /// Errors that occur during error handling
    pub(crate) incidental: Vec<Error>,
}

/// A comprehensive structural error type, intended exclusively for human
/// consumption.
#[derive(Debug, Clone)]
pub struct Error(pub(crate) Box<Error_>);

pub(crate) struct RenderNodeBranch<'a> {
    pub(crate) title: Cow<'a, str>,
    pub(crate) children: Vec<RenderNode<'a>>,
}

pub(crate) enum RenderNode<'a> {
    KVLeaf {
        key: &'a str,
        value: &'a String,
    },
    Branch(RenderNodeBranch<'a>),
}

impl Error {
    pub fn from(x: impl Display) -> Error {
        return Error(Box::new(Error_ {
            message: x.to_string(),
            attrs: HashMap::new(),
            context: vec![],
            causes: vec![],
            incidental: vec![],
        }));
    }

    /// Extend the base error with a new incidental (occurred while handling the base
    /// error) error.  Use like `e.also(log, new_e);`.
    pub fn also(mut self, incidental: Error) -> Error {
        self.0.incidental.push(incidental);
        return self;
    }

    pub(crate) fn build_render_nodes<'a>(&'a self, seen_contexts: &HashSet<*const Log_>) -> RenderNodeBranch<'a> {
        let message;
        let mut children = vec![];
        let mut sub_seen_contexts = seen_contexts.clone();
        message = self.0.message.to_string();
        let mut seen_attrs = HashSet::new();
        for a in &self.0.attrs {
            if !seen_attrs.insert(*a.0) {
                continue;
            }
            children.push(RenderNode::KVLeaf {
                key: a.0,
                value: a.1,
            })
        }
        for context in &self.0.context {
            let mut at = Some(context);
            while let Some(at1) = at {
                let addr = at1.0.as_ref() as *const Log_;
                if seen_contexts.contains(&addr) {
                    break;
                }
                for a in &at1.0.attrs {
                    if !seen_attrs.insert(*a.0) {
                        continue;
                    }
                    children.push(RenderNode::KVLeaf {
                        key: a.0,
                        value: a.1,
                    })
                }
                sub_seen_contexts.insert(at1.0.as_ref());
                at = at1.0.parent.as_ref();
            }
        }
        if self.0.causes.len() > 0 {
            children.push(RenderNode::Branch(RenderNodeBranch {
                title: "Caused by:".into(),
                children: self
                    .0
                    .causes
                    .iter()
                    .map(|x| RenderNode::Branch(x.build_render_nodes(&sub_seen_contexts)))
                    .collect(),
            }));
        }
        if self.0.incidental.len() > 0 {
            children.push(RenderNode::Branch(RenderNodeBranch {
                title: "Incidentally:".into(),
                children: self
                    .0
                    .incidental
                    .iter()
                    .map(|x| RenderNode::Branch(x.build_render_nodes(&sub_seen_contexts)))
                    .collect(),
            }));
        }
        return RenderNodeBranch {
            title: message.into(),
            children: children,
        };
    }

    /// Return a new error adding a simple string message as a layer of context.
    pub fn context(self, message: impl ToString) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![],
            causes: vec![self],
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context with a message and attributes.
    pub fn context_with(
        self,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: new_attrs,
            context: vec![],
            causes: vec![self],
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context including all attributes in the
    /// provided log along with the specified message.
    pub fn stack_context(self, log: &Log, message: impl ToString) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![log.clone()],
            causes: vec![self],
            incidental: vec![],
        }));
    }

    /// Return a new error adding a layer of context including all attributes in the
    /// provided log along with the specified message and new attributes.
    pub fn stack_context_with(
        self,
        log: &Log,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: new_attrs,
            context: vec![log.clone()],
            causes: vec![self],
            incidental: vec![],
        }));
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = RenderNode::Branch(self.build_render_nodes(&HashSet::new()));
        let mut stack = vec![(true, 0, &node)];
        while let Some((descending, index, top)) = stack.pop() {
            match top {
                RenderNode::KVLeaf { key, value } => {
                    if index > 0 {
                        f.write_str(",")?;
                    }
                    f.write_str(" ")?;
                    f.write_str(key)?;
                    f.write_str(" = ")?;
                    f.write_str(&value)?;
                },
                RenderNode::Branch(b) => {
                    if descending {
                        f.write_str(&b.title)?;
                        f.write_str(" [")?;
                        stack.extend(b.children.iter().enumerate().rev().map(|(i, e)| (true, i, e)));
                    } else {
                        f.write_str(" ]")?;
                    }
                },
            }
        }
        return Ok(());
    }
}

impl<T: std::error::Error> From<T> for Error {
    fn from(value: T) -> Self {
        return Error::from(value);
    }
}

pub(crate) fn log(body_color: Style, level_color: Style, level_text: &str, node: RenderNodeBranch) {
    let highlight_style = Style::new().for_stderr().blue();
    let dark_style = Style::new().for_stderr().dim();
    let mut out = String::new();
    let mut stack = node.children.iter().rev().map(|e| (0, e)).collect::<Vec<_>>();
    while let Some((indent_count, top)) = stack.pop() {
        let indent = "  ".repeat(indent_count);
        match top {
            RenderNode::KVLeaf { key, value } => {
                let key = format!("- {} = ", key);
                for line in wrap(
                    &value,
                    Options::with_termwidth()
                        .initial_indent(&format!("{}{}", indent, key))
                        .subsequent_indent(&format!("{}{}", indent, " ".repeat(measure_text_width(&key)))),
                ) {
                    out.push_str(&dark_style.apply_to(line).to_string());
                    out.push('\n');
                }
            },
            RenderNode::Branch(b) => {
                for line in wrap(
                    &format!("{}", b.title),
                    Options::with_termwidth().initial_indent(&indent).subsequent_indent(&indent),
                ) {
                    out.push_str(&highlight_style.apply_to(line).to_string());
                    out.push('\n');
                }
                stack.extend(b.children.iter().rev().map(|e| (indent_count + 1, e)));
            },
        }
    }
    eprint!(
        "{} {}: {}\n{}",
        body_color.apply_to(Local::now().to_rfc3339()),
        level_color.apply_to(level_text),
        node.title,
        out
    );
}

/// A store of context with methods for logging and creating errors expressing that
/// context.
#[derive(Clone, Debug)]
pub struct Log(Arc<Log_>);

#[derive(Debug)]
pub(crate) struct Log_ {
    pub(crate) parent: Option<Log>,
    pub(crate) attrs: HashMap<&'static str, String>,
    pub(crate) log_from: Option<Level>,
}

impl Default for Log {
    fn default() -> Self {
        return Self(Arc::new(Log_ {
            parent: None,
            attrs: HashMap::new(),
            log_from: None,
        }));
    }
}

impl Log {
    /// Create a new non-rooted (non-logging) context for gathering contextual
    /// attributes for adding to errors.
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn new_root(log_from: Level) -> Self {
        return Self(Arc::new(Log_ {
            parent: None,
            attrs: HashMap::new(),
            log_from: Some(log_from),
        }));
    }

    /// Create a new `Log` that inherits attributes from the base logging context.  Use
    /// like `let new_log = log.fork(ea!(newkey = newvalue, ...));`.
    pub fn fork(&self, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Self {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Self(Arc::new(Log_ {
            parent: Some(self.clone()),
            attrs: new_attrs,
            log_from: self.0.log_from,
        }));
    }

    /// Like `fork` but also increase the minimum log level.
    pub fn fork_with_log_from(&self, log_from: Level, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Self {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Self(Arc::new(Log_ {
            parent: Some(self.clone()),
            attrs: new_attrs,
            log_from: self.0.log_from.map(|x| x.max(log_from)),
        }));
    }

    /// Log a message.  The message will only be rendered and output if any of the
    /// specified flags are set.
    pub fn log(&self, level: Level, message: impl ToString) {
        self.log_with(level, message, ea!());
    }

    fn should_log(&self, level: Level) -> Option<Level> {
        // Not rooted/context only
        let Some(log_from) = self.0.log_from else {
            return None;
        };
        if level < log_from {
            return None;
        }
        return Some(level);
    }

    /// Log a message.  The attributes will only be evaluated and the message will only
    /// be rendered and output if any of the specified flags are set.
    pub fn log_with(
        &self,
        level: Level,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        let Some(level) = self.should_log(level) else {
            return;
        };
        self.log_err(level, self.err_with(message, attrs));
    }

    pub fn log_err(&self, level: Level, mut e: Error) {
        let Some(level) = self.should_log(level) else {
            return;
        };
        let body_style;
        let label_style;
        let label;
        match level.0 {
            1 => {
                body_style = Style::new().for_stderr().black().bright();
                label_style = Style::new().for_stderr().black().bright();
                label = "DEBUG";
            },
            2 => {
                body_style = Style::new().for_stderr().black();
                label_style = Style::new().for_stderr().black();
                label = "INFO";
            },
            3 => {
                body_style = Style::new().for_stderr().black();
                label_style = Style::new().for_stderr().yellow();
                label = "WARN";
            },
            4 => {
                body_style = Style::new().for_stderr().black();
                label_style = Style::new().for_stderr().red();
                label = "ERROR";
            },
            _ => unreachable!(),
        }
        e.0.context.push(self.clone());
        log(body_style, label_style, label, e.build_render_nodes(&HashSet::new()));
    }

    /// Create a new error including the attributes in this logging context.
    pub fn err(&self, message: impl ToString) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![self.clone()],
            causes: vec![],
            incidental: vec![],
        }));
    }

    /// Create a new error including the attributes in this logging context and merging
    /// additional attributes.
    pub fn err_with(&self, message: impl ToString, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: new_attrs,
            context: vec![self.clone()],
            causes: vec![],
            incidental: vec![],
        }));
    }

    /// Create an error from multiple errors, attaching the Log's attributes.
    pub fn agg_err(&self, message: impl ToString, errs: Vec<Error>) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![self.clone()],
            causes: errs,
            incidental: vec![],
        }));
    }

    /// Create an error from multiple errors, attaching the Log's attributes and
    /// additional attributes.
    pub fn agg_err_with(
        &self,
        message: impl ToString,
        errs: Vec<Error>,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error {
        let mut new_attrs = HashMap::new();
        attrs(&mut new_attrs);
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: new_attrs,
            context: vec![self.clone()],
            causes: errs,
            incidental: vec![],
        }));
    }
}
