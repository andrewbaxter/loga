Logging and error management are tightly coupled. When both logging and creating errors, you need to provide context (what the program was doing, with what, where, when).

This crate handles provides interconnected, structured logging and error handling. The logger object manages this state and provides it when logging and creating errors. Rather than add context while unwinding from an error, the context is established beforehand at various scopes by refining old logger objects with additional attributes (though context can be added afterwards as well).

## Goals

The goals fo this crate are ease of use and expressivitiy, both for developing code and people reading the errors/log messages.

Optimization will be considered as necessary as far as it doesn't impact ease of use and expressivity.

## Event structure

Events (errors and log messages) have a tree structure, with the following dimensions:

- Attributes at the current level of context
- One or more errors that this error adds context to (causes)
- One or more errors that occurred while trying to handle this error (incidental)

The errors are intended _only_ for human consumption. Any information that may need to be handled programmatically should be in a non-error return.

## Usage tips

In non-logging functions or objects that may be shared in multiple contexts, rather than receive a logger from the caller it may be simpler to start a new (blank) Log tree internally, or just use `.context`. The caller can later add attributes using `.log_context`.

When you raise an error, you typically only want to add context from a log only once per independent `Log` tree, at the deepest level of specificity (the logger closest to where you raise the error). This happens when you do `log.new_err` or call `.log_context` on an error from outside to bring it into the logging system. At all other locations you can use `.context`. This will avoid unnecessarily duplicating attributes.

## Notes

Currently logging only happens to stderr. Adding more log destinations and formats in the future would be nice.
