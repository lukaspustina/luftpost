use config::Smtp;
use measurement::Measurement;
use lettre::email::{Email, EmailBuilder};
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::stub::StubEmailTransport;
use lettre::transport::file::FileEmailTransport;

error_chain!{
    errors {
    }
    foreign_links {
        FileTransportError(::lettre::transport::file::error::Error);
        SmtpTransportError(::lettre::transport::smtp::error::Error);
        StubTransportError(::lettre::transport::stub::error::Error);
        EmailFormatError(::lettre::email::error::Error);
    }
}

pub enum Transport {
    File(FileEmailTransport),
    Smtp(SmtpTransport),
    Stub(StubEmailTransport)
}

pub fn create_transport(smtp: Smtp) -> Result<Transport> {
    unimplemented!();
}

pub fn mail_measurement(measurement: Measurement, transport: &mut Transport) -> Result<()> {
    let email = EmailBuilder::new()
        .to(&measurement.sensor.e_mail_addr.unwrap()[..])
        .from("user@localhost")
        .subject(&measurement.sensor.e_mail_subject.unwrap())
        .body("Hello World!")
        .build()?;
    transport.send(email)
}

impl Transport {
    fn send(&mut self, email: Email) -> Result<()> {
        match *self {
            Transport::File(ref mut file) => file.send(email).map_err(|e| e.into()).map(|_| ()),
            Transport::Smtp(ref mut smtp) => smtp.send(email).map_err(|e| e.into()).map(|_| ()),
            Transport::Stub(ref mut stub) => stub.send(email).map_err(|e| e.into()).map(|_| ()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use measurement::Value;
    use lettre::transport::stub::StubEmailTransport;
    use sensor::Sensor;

    #[test]
    fn mail_measurement_okay() -> () {
        let sensor = Sensor {
            name: "A Sensor".to_string(),
            uri: "http://localhost".to_string(),
            threshold_pm10: Some(10.0),
            threshold_pm2: Some(2.0),
            e_mail_addr: Some("test@example.com".to_string()),
            e_mail_subject: Some("PM Alarm".to_string()),
            e_mail_condition: None,
        };
        let mut data_values = Vec::new();
        data_values.push(Value::SDS_P1(7.87f32));
        data_values.push(Value::SDS_P2(3.17f32));
        data_values.push(Value::TEMPERATURE(18.90f32));
        data_values.push(Value::HUMIDITY(49.10f32));
        data_values.push(Value::SAMPLES(739514f32));
        data_values.push(Value::MIN_MICRO(192f32));
        data_values.push(Value::MAX_MICRO(27599f32));
        data_values.push(Value::SIGNAL(-73f32));
        let measurement = Measurement {
            sensor: sensor,
            software_version: "NRZ-2017-089".to_string(),
            data_values: data_values,
        };
        let mut transport = Transport::Stub(StubEmailTransport);

        let res = mail_measurement(measurement, &mut transport);

        assert!(res.is_ok());
    }
}
