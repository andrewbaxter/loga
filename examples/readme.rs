struct Resp {
    body: Vec<u8>,
    code: u16,
}

fn http_req(_url: &str) -> Result<Resp, loga::Error> {
    panic!();
}

fn launch_satellite(_body: Vec<u8>) -> Result<(), loga::Error> {
    panic!();
}

fn shutdown_server() -> Result<(), loga::Error> {
    panic!();
}

use loga::{
    StandardFlags,
    ea,
    ResultContext,
    ErrContext,
};

const WARN: StandardFlags = StandardFlags::Warning;
const INFO: StandardFlags = StandardFlags::Info;

fn main1() -> Result<(), loga::Error> {
    // All errors stacked from this will have "system = main"
    let log = &loga::Log::<StandardFlags>::new().with_flags(&[WARN, INFO]).fork(ea!(system = "main"));

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
