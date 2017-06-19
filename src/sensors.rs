use measurements;
use measurements::{Measurement, wire_to_measurement};
use measurements::wire::decode_json_to_measurement;
use futures::{Future, Stream};
use hyper::Client;
use hyper::client::{FutureResponse, HttpConnector};
use std::str;
use tokio_core::reactor::Core;

error_chain! {
    errors {

    }
    links {
        MeasurementTransformationFailed(measurements::Error, measurements::ErrorKind);
        MeasurementDecodingFailed(measurements::wire::Error, measurements::wire::ErrorKind);
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

pub fn read_measurement(response: FutureResponse) -> Box<Future<Item=Measurement, Error=Error>> {
    let m = response.and_then(|res| {
        res.body().concat2()
    }).map(|body| {
        let json = str::from_utf8(&body)?;
        let wire = decode_json_to_measurement(json)?;
        wire_to_measurement(wire).map_err(|e| e.into())
    }).map_err(|e| e.into())
    .and_then(|x| x);
    Box::new(m)
}

