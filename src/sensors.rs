use measurements;
use measurements::{Measurement, measurement_from_json};
use futures::{Future, Stream};
use hyper::Client;
use hyper::client::{FutureResponse, HttpConnector};
use std::str;
use tokio_core::reactor::Core;

error_chain! {
    errors {

    }
    links {
        ReadingMeasurementFailed(measurements::Error, measurements::ErrorKind);
    }
    foreign_links {
        Fmt(::std::str::Utf8Error);
        Io(::std::io::Error);
        Hyper(::hyper::Error);
    }
}

pub fn create_client(core: &mut Core) -> Client<HttpConnector> {
    Client::new(&core.handle())
}

pub fn read_measurement(
    sensor_name: String,
    response: FutureResponse,
) -> Box<Future<Item = Measurement, Error = Error>> {
    let m = response
        .and_then(|res| res.body().concat2())
        .map(|body| {
            let json = str::from_utf8(&body)?;
            measurement_from_json(sensor_name, json).map_err(|e| e.into())
        })
        .map_err(|e| e.into())
        .and_then(|x| x);
    Box::new(m)
}
