Logging and error management are tightly coupled, and this does both. When both logging and creating errors, you need to provide context (what the program was doing, with what, where, when). In this crate, the logger object manages this state and provides it when logging and creating errors. Rather than add context while unwinding from an error, the context is established beforehand by refining new logger objects (though context can be added afterwards as well).

Errors have a tree structure, with a single error encompassing possibly multiple cause errors and incidental errors that arose during error processing.

The goals are ease of use and expressivitiy, both for developing code and people reading the errors/log messages. Optimization will be considered as necessary as far as it doesn't impact the prior goal.

The errors are intended _only_ for human consumption. Any information that may need to be handled programmatically should be in a non-error return.

Currently logging only happens to stderr. Adding more log destinations and formats in the future would be nice.
