Logging and error management are tightly coupled. When both logging and creating errors, you need to provide context (what the program was doing, with what, where, when).

This crate handles provides interconnected, structured logging and error handling. The logger object manages this state and provides it when logging and creating errors. Rather than add context while unwinding from an error, the context is established beforehand at various scopes by refining old logger objects with additional attributes (though context can be added afterwards as well).

```rust
use loga::{
    common::StandardFlags,
    ea,
    ResultContext,
    ErrContext,
};

const WARN: StandardFlags = StandardFlags::WARN;
const INFO: StandardFlags = StandardFlags::INFO;
const DEBUG: StandardFlags = StandardFlags::DEBUG;

fn main1() -> Result<(), loga::Error> {
    // All errors stacked from this will have "system = main"
    let log = &loga::new().fork(ea!(system = "main"));

    // Convert the error result to `loga::Error`, add all the logger's attributes, add
    // a message, and add additional attributes.
    let res =
        http_req(
            "https://example.org",
        ).stack_context_with(log, "Example.org is down", ea!(process = "get_weather"))?;
    match launch_satellite(res.body) {
        Ok(_) => (),
        Err(e) => {
            let e = e.stack_context(log, "Failed to launch satellite");
            if let Err(e2) = shutdown_server() {
                // Attach incidental errors
                return Err(e.also(e2.into()));
            }
            return Err(e);
        },
    }
    if res.code == 295 {
        return Err(loga::err("Invalid response"));
    }
    log.log(INFO, "Obtained weather");
    return Ok(());
}

fn main() {
    loga::init_flags(WARN | INFO);
    match main1() {
        Ok(_) => (),
        Err(e) => loga::fatal(e),
    }
}
```

## Goals

The goals fo this crate are ease of use and expressivitiy, both for developing code and people reading the errors/log messages.

Optimization will be considered as necessary as far as it doesn't impact ease of use and expressivity.

## Event structure

Events (errors and log messages) have a tree structure, with the following dimensions:

- Attributes at the current level of context
- One or more errors that this error adds context to (causes)
- One or more errors that occurred while trying to handle this error (incidental)

The errors are intended _only_ for human consumption. Any information that may need to be handled programmatically should be in a non-error return.

## Flags

Unlike traditional loggers which have a linear scale of levels, sometimes multiplied by different "loggers" or additional dimensions, this library uses a set of additive flags. If any of the flags in the log call are set in the flags provided in `init_flags` then the log message will be rendered and displayed, otherwise it will be dropped.

To keep things simple you can use `StandardFlags` with `DEBUG`, `INFO`, `WARN`, etc. levels.

If you have multiple subsystems each with their own levels, you could for instance have `GC_DEBUG`, `GC_INFO`, `GC_WARN`, `HTTP_DEBUG`, `HTTP_INFO`, etc. This is hopefully a simple mechanism for allowing pinpoint log control.

Flags only affect logging, not errors.

## Usage tips

You may want to alias `new()` to hard code the flags of your choice.

In non-logging functions or objects that may be shared in multiple contexts, rather than receive a logger from the caller it may be simpler to start a new (blank) Log tree internally, or just use `.context`. The caller can later add attributes using `.log_context`.

When you raise an error, you typically only want to add context from a log only once per independent `Log` tree, at the deepest level of specificity (the logger closest to where you raise the error). This happens when you do `log.new_err` or call `.log_context` on an error from outside to bring it into the logging system. At all other locations you can use `.context`. This will avoid unnecessarily duplicating attributes.

## Notes

Currently logging only happens to stderr. Adding more log destinations and formats in the future would be nice.
