use std::collections::HashMap;
use crate::{
    types::{
        Error,
        Log,
        FullError,
        Error_,
        Error_2,
    },
    ea,
    entry::{
        err,
        err_with,
    },
    Flags,
};

/// A trait adding helper methods to standard errors to convert to `loga::Error`.
pub trait ErrContext {
    /// Add a simple context string onto an error, converting it to `loga::Error` in
    /// the process.
    fn context(self, message: &'static str) -> Error;

    /// Add a simple context string and attributes pairs onto an error, converting it
    /// to `loga::Error` in the process.
    fn context_with(self, message: &'static str, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error;

    /// Add attributes from the log as well as a simple context string to an error,
    /// converting it to `loga::Error` in the process.
    fn stack_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Error;

    /// Add attributes from the log as well as the specified attributes and a simple
    /// context string to an error, converting it to `loga::Error` in the process.
    fn stack_context_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
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

    fn stack_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Error {
        return Error(Box::new(Error_ {
            inner: Error_2::Full(FullError {
                message: message,
                attrs: log.attrs.clone(),
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }

    fn stack_context_with<
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
                causes: vec![self.into()],
            }),
            incidental: vec![],
        }));
    }
}

/// A trait adding helper methods to `Result` to convert to
/// `Result<_, loga::Error>`.
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
    fn stack_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Result<O, Error>;

    /// If the value is Err/None, add attributes from the log as well as the specified
    /// attributes and a simple context string to an error, converting it to
    /// `loga::Error` in the process.
    fn stack_context_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;

    // If the value is Err/None, consume it, logging it with the additional context
    // message.
    fn log<F: Flags>(self, log: &Log<F>, flags: F, message: &'static str);

    // If the value is Err/None, consume it, logging it with the additional context
    // message and attributes.
    fn log_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        flags: F,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    );
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

    fn stack_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.stack_context(log, message)),
        }
    }

    fn stack_context_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.stack_context_with(log, message, attrs)),
        }
    }

    fn log<F: Flags>(self, log: &Log<F>, flags: F, message: &'static str) {
        if let Err(e) = self.context(message) {
            log.log_err(flags, e);
        }
    }

    fn log_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        flags: F,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        if let Err(e) = self.context_with(message, attrs) {
            log.log_err(flags, e);
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

    fn stack_context<F: Flags>(self, log: &Log<F>, message: &'static str) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.err(message)),
        }
    }

    fn stack_context_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.err_with(message, attrs)),
        }
    }

    fn log<F: Flags>(self, log: &Log<F>, flags: F, message: &'static str) {
        if self.is_none() {
            log.log_err(flags, err("No value").context(message));
        }
    }

    fn log_with<
        F: Flags,
    >(
        self,
        log: &Log<F>,
        flags: F,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        if self.is_none() {
            log.log_err(flags, err("No value").context_with(message, attrs));
        }
    }
}
