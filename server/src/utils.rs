use log::error;
use std::str::FromStr;

pub fn handle_result<T, E: std::fmt::Debug>(result: Result<T, E>) -> Option<T> {
    if let Err(err) = result.as_ref() {
        error!("{:?}", err);
    }

    result.ok()
}

pub fn parse_config_field<F: FromStr>(field: &str) -> Option<F> {
    std::fs::read_to_string("./server.cfg")
        .ok()
        .and_then(|inner| {
            inner
                .lines()
                .find(|line| line.starts_with(field))
                .map(|borrow| borrow.to_string())
                .and_then(|bind| {
                    bind.split(" ")
                        .skip(1)
                        .next()
                        .map(|borrow| borrow.to_string())
                })
        })
        .and_then(|addr| addr.parse().ok())
}
