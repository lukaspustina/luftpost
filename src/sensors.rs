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

#[cfg(test)]
mod test {
    use super::*;
    use measurements::ValueType;

    #[test]
    fn read_measurement_local() -> () {
        let mut core = Core::new().unwrap();

        let uri = "http://feinstaub/data.json".parse().unwrap();
        let request = create_client(&mut core).get(uri);
        let work = read_measurement(request);
        let res = core.run(work).unwrap();

        assert!(res.data_values.contains_key(&ValueType::SDS_P1));
        assert!(res.data_values.contains_key(&ValueType::SDS_P2));
        assert!(res.data_values.contains_key(&ValueType::TEMPERATURE));
        assert!(res.data_values.contains_key(&ValueType::HUMIDITY));
        assert!(res.data_values.contains_key(&ValueType::SAMPLES));
        assert!(res.data_values.contains_key(&ValueType::MIN_MICRO));
        assert!(res.data_values.contains_key(&ValueType::MAX_MICRO));
        assert!(res.data_values.contains_key(&ValueType::SIGNAL));
    }
}
