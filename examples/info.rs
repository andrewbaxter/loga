use loga::{
    ea,
    Log,
};

fn main() {
    let log = Log::new_root(loga::INFO);
    log.log(loga::INFO, "Hello");
    log.log_with(loga::INFO, "Hello", ea!(xyz = "abc", system = "primary"));
    log.log_err(
        loga::INFO,
        loga::err_with(
            "This is a sub-error",
            ea!(system = "secondary"),
        ).context_with("Got an error", ea!(context = "Additional details")),
    );
    log.log_err(
        loga::INFO,
        loga::agg_err_with(
            "This is a sub-error",
            vec![loga::err("Problem 1"), loga::err_with("Problem 2", ea!(client = "a:b:c:d"))],
            ea!(system = "secondary"),
        ).context_with("Got an error", ea!(context = "Additional details")),
    );
    let log2 = Log::new_root(loga::INFO).fork(ea!(logger = "log2"));
    log2.log_err(
        loga::INFO,
        loga::agg_err_with(
            "This is a sub-error",
            vec![log2.err("Problem 1"), log2.err_with("Problem 2", ea!(client = "a:b:c:d"))],
            ea!(system = "secondary"),
        ).context_with("Logging with logger details", ea!(context = "Additional details")),
    );
}
