use config;
use measurements;
use measurements::Measurement;
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

#[derive(Debug, Deserialize)]
#[derive(PartialEq)]
pub struct Sensor {
    pub name: String,
    pub uri: String,
    pub threshold_pm10: Option<f32>,
    pub threshold_pm2: Option<f32>,
    pub e_mail_addr: Option<String>,
    pub e_mail_subject: Option<String>,
    #[serde(default = "Vec::new")]
    pub e_mail_condition: Vec<config::EmailCondition>,
}

impl Sensor {
    pub fn new<T: Into<String>>(name: T, uri: T) -> Sensor {
        Sensor {
            name: name.into(),
            uri: uri.into(),
            threshold_pm10: None,
            threshold_pm2: None,
            e_mail_addr: None,
            e_mail_subject: None,
            e_mail_condition: Vec::new(),
        }
    }

    pub fn read_measurement(
        self: Self,
        response: FutureResponse,
    ) -> Box<Future<Item = Measurement, Error = Error>> {
        let m = response
            .and_then(|res| res.body().concat2())
            .map(|body| {
                let json = str::from_utf8(&body)?;
                Measurement::from_json(self, json).map_err(|e| e.into())
            })
            .map_err(|e| e.into())
            .and_then(|x| x);
        Box::new(m)
    }
}

pub fn create_client(core: &mut Core) -> Client<HttpConnector> {
    Client::new(&core.handle())
}
