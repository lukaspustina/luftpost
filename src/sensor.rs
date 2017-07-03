use config;
use measurement::{self, Measurement};
use futures::{Future, Stream};
use hyper::Client;
use hyper::client::{FutureResponse, HttpConnector};
use std::str;
use tokio_core::reactor::Core;

error_chain! {
    errors {

    }
    links {
        ReadingMeasurementFailed(measurement::Error, measurement::ErrorKind);
    }
    foreign_links {
        Fmt(::std::str::Utf8Error);
        Io(::std::io::Error);
        Hyper(::hyper::Error);
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct Sensor {
    pub name: String,
    pub id: String,
    pub ui_uri: String,
    pub data_uri: String,
    pub threshold_pm10: Option<f32>,
    pub threshold_pm2: Option<f32>,
    pub notification_condition: Option<config::NotificationCondition>,
}

impl Sensor {
    pub fn new<T: Into<String>>(name: T, id: T, ui_uri: T, data_uri: T) -> Sensor {
        Sensor {
            name: name.into(),
            id: id.into(),
            ui_uri: ui_uri.into(),
            data_uri: data_uri.into(),
            threshold_pm10: None,
            threshold_pm2: None,
            notification_condition: None,
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

pub fn create_sensor_reader(core: &mut Core) -> Client<HttpConnector> {
    Client::new(&core.handle())
}
