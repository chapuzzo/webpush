use std::error::Error;

use isahc::{
    config::{Configurable, SslOption},
    send, ReadResponseExt, Request, RequestExt,
};

const URI: &str = "https://updates.push.services.mozilla.com";

fn main() -> Result<(), Box<dyn Error>> {
    let mut w = Request::get(URI)
        .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS)
        .body(())?
        .send()?;

    println!("{:?}", w.text()?);

    let x = isahc::HttpClientBuilder::new();
    let x = x.build()?;
    // let w = isahc::config::SslOption
    let mut result = x.get(URI)?;

    println!("{:?}", &result);
    println!("{}", result.text()?);

    Ok(())
}
