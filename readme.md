Errors and log messages are closely related - both are events, where logs are output in realtime (to a terminal or remote logger) without affecting execution flow and errors are passed up in lieu of a successful result for another context to either log or discard. Both provide information what was happening at the time of the event, including context up the stack (what was happening, as part of what larger actions, and with what relationships to other entities).

With this library you build a tree of `Log` objects to store context at multiple levels and the `Log` object has methods for logging and creating errors which contain the full tree of context.

```rust
use loga::{
    ea,
    ResultContext,
    INFO,
};

fn main1() -> Result<(), loga::Error> {
    // All errors stacked from this will have "system = main"
    let log = &loga::Log::new_root(INFO).fork(ea!(system = "main"));

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
    match main1() {
        Ok(_) => (),
        Err(e) => loga::fatal(e),
    }
}
```

## Goals

The goal of this crate is to make it easy to produce clear and informative log messages and errors with full context. To do that creating errors and log messages as well as adding context must be easy.

Optimization will be considered as necessary as far as it doesn't impact ease of use and expressivity.

## Event structure

Events (errors and log messages) also have a tree structure, with the following dimensions:

- Attributes at the current level of context
- One or more errors that this error adds context to (causes)
- One or more errors that occurred while trying to handle this error (incidental)

The errors are intended _only_ for human consumption. Any information that may need to be handled programmatically should be in a non-error return.

## Usage tips

In non-logging functions or objects that may be shared in multiple contexts, rather than receive a logger from the caller it may be simpler to start a new (blank) Log tree internally, or just use `.context`. The caller can later root the context using `.stack_context` or the logger's context will naturally be added in `log.log_err`.

## Notes

Currently logging is written to stderr only. Adding more log destinations and formats in the future would be nice.
