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
        new_err,
        new_err_with,
    },
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
            None => Err(new_err(message)),
        }
    }

    fn context_with(
        self,
        message: &'static str,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(new_err_with(message, attrs)),
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
