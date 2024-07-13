use std::{
    collections::HashMap,
};
use crate::{
    entry::{
        err,
        err_with,
    },
    types::{
        Error,
        Error_,
    },
    Level,
    Log,
};

/// A trait adding helper methods to standard errors to convert to `loga::Error`.
pub trait ErrContext {
    /// Add a simple context string onto an error, converting it to `loga::Error` in
    /// the process.
    fn context(self, message: impl ToString) -> Error;

    /// Add a simple context string and attributes pairs onto an error, converting it
    /// to `loga::Error` in the process.
    fn context_with(self, message: impl ToString, attrs: impl Fn(&mut HashMap<&'static str, String>) -> ()) -> Error;

    /// Add attributes from the log as well as a simple context string to an error,
    /// converting it to `loga::Error` in the process.
    fn stack_context(self, log: &Log, message: impl ToString) -> Error;

    /// Add attributes from the log as well as the specified attributes and a simple
    /// context string to an error, converting it to `loga::Error` in the process.
    fn stack_context_with(
        self,
        log: &Log,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Error;
}

impl<T: Into<Error>> ErrContext for T {
    fn context(self, message: impl ToString) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![],
            causes: vec![self.into()],
            incidental: vec![],
        }));
    }

    fn context_with(
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
            causes: vec![self.into()],
            incidental: vec![],
        }));
    }

    fn stack_context(self, log: &Log, message: impl ToString) -> Error {
        return Error(Box::new(Error_ {
            message: message.to_string(),
            attrs: HashMap::new(),
            context: vec![log.clone()],
            causes: vec![self.into()],
            incidental: vec![],
        }));
    }

    fn stack_context_with(
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
            causes: vec![self.into()],
            incidental: vec![],
        }));
    }
}

/// A trait adding helper methods to `Result` to convert to
/// `Result<_, loga::Error>`.
pub trait ResultContext<O> {
    /// If the value is Err/None, add a simple context string onto an error, converting
    /// it to `loga::Error` in the process.
    fn context(self, message: impl ToString) -> Result<O, Error>;

    /// If the value is Err/None, add a simple context string and attributes pairs onto
    /// an error, converting it to `loga::Error` in the process.
    fn context_with(
        self,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;

    /// If the value is Err/None, add attributes from the log as well as a simple
    /// context string to an error, converting it to `loga::Error` in the process.
    fn stack_context(self, log: &Log, message: impl ToString) -> Result<O, Error>;

    /// If the value is Err/None, add attributes from the log as well as the specified
    /// attributes and a simple context string to an error, converting it to
    /// `loga::Error` in the process.
    fn stack_context_with(
        self,
        log: &Log,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error>;

    /// If this is an error and the argument result is an error, attach the argument
    /// result to this error as an incidental error. If this isn't an error and the
    /// argument result is an error, return the argument result alone.
    fn also<O2, E: Into<Error>>(self, r: Result<O2, E>) -> Result<O, Error>;

    // If the value is Err/None, consume it, logging it with the additional context
    // message.
    fn log(self, log: &Log, level: Level, message: impl ToString);

    // If the value is Err/None, consume it, logging it with the additional context
    // message and attributes.
    fn log_with(
        self,
        log: &Log,
        level: Level,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    );
}

impl<O, E: Into<Error>> ResultContext<O> for Result<O, E> {
    fn context(self, message: impl ToString) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.context(message)),
        }
    }

    fn context_with(
        self,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.context_with(message, attrs)),
        }
    }

    fn stack_context(self, log: &Log, message: impl ToString) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.stack_context(log, message)),
        }
    }

    fn stack_context_with(
        self,
        log: &Log,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(e.stack_context_with(log, message, attrs)),
        }
    }

    fn also<O2, E2: Into<Error>>(self, r: Result<O2, E2>) -> Result<O, Error> {
        match self {
            Ok(o) => match r {
                Ok(_) => {
                    return Ok(o);
                },
                Err(e2) => {
                    return Err(e2.into());
                },
            },
            Err(e) => match r {
                Ok(_) => return Err(e.into()),
                Err(e2) => return Err(e.into().also(e2.into())),
            },
        }
    }

    fn log(self, log: &Log, level: Level, message: impl ToString) {
        if let Err(e) = self.context(message) {
            log.log_err(level, e);
        }
    }

    fn log_with(
        self,
        log: &Log,
        level: Level,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        if let Err(e) = self.context_with(message, attrs) {
            log.log_err(level, e);
        }
    }
}

impl<O> ResultContext<O> for Option<O> {
    fn context(self, message: impl ToString) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(err(message)),
        }
    }

    fn context_with(
        self,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(err_with(message, attrs)),
        }
    }

    fn stack_context(self, log: &Log, message: impl ToString) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.err(message)),
        }
    }

    fn stack_context_with(
        self,
        log: &Log,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) -> Result<O, Error> {
        match self {
            Some(x) => Ok(x),
            None => Err(log.err_with(message, attrs)),
        }
    }

    fn also<O2, E2: Into<Error>>(self, r: Result<O2, E2>) -> Result<O, Error> {
        match self {
            Some(o) => match r {
                Ok(_) => {
                    return Ok(o);
                },
                Err(e2) => {
                    return Err(e2.into());
                },
            },
            None => match r {
                Ok(_) => return Err(err("No value")),
                Err(e2) => return Err(err("No value").also(e2.into())),
            },
        }
    }

    fn log(self, log: &Log, level: Level, message: impl ToString) {
        if self.is_none() {
            log.log_err(level, err("No value").context(message));
        }
    }

    fn log_with(
        self,
        log: &Log,
        level: Level,
        message: impl ToString,
        attrs: impl Fn(&mut HashMap<&'static str, String>) -> (),
    ) {
        if self.is_none() {
            log.log_err(level, err("No value").context_with(message, attrs));
        }
    }
}
