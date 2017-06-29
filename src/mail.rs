use config::Smtp;
use handlebars::{Handlebars, RenderError, RenderContext, Helper};
use measurement::Measurement;
use lettre::email::{Email, EmailBuilder};
use lettre::transport::EmailTransport;
use lettre::transport::smtp::{SecurityLevel, SmtpTransport, SmtpTransportBuilder};
use lettre::transport::smtp::SUBMISSION_PORT;
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
        TemplateError(::handlebars::TemplateRenderError);
        IoError(::std::io::Error);
    }
}

pub enum Transport {
    File(FileEmailTransport),
    Smtp(SmtpTransport),
    Stub(StubEmailTransport)
}

pub struct Mailer<'a> {
    pub transport: Transport,
    pub from_addr: &'a str,
    pub text_template: &'a str,
    pub html_template: &'a str,
}

impl<'a> Mailer<'a> {
    pub fn create_mailer(smtp: &'a Smtp) -> Result<Mailer<'a>> {
        let mut builder = SmtpTransportBuilder::new(
            (&smtp.server[..], smtp.port.unwrap_or_else(|| SUBMISSION_PORT))).unwrap()
            .hello_name("my.hostname.tld")
            .security_level(SecurityLevel::Opportunistic)
            .smtp_utf8(true)
            .connection_reuse(true);
        if smtp.username.is_some() && smtp.password.is_some() && smtp.auth_mechanism.is_some() {
            builder = builder
                .credentials(smtp.username.as_ref().unwrap(), smtp.password.as_ref().unwrap())
                .authentication_mechanism(smtp.auth_mechanism.unwrap());
        }
        let transport = builder.build();

        let mailer = Mailer {
            transport: Transport::Smtp(transport),
            from_addr: &smtp.sender,
            text_template: &smtp.text_template,
            html_template: &smtp.html_template,
        };

        Ok(mailer)
    }

    pub fn mail_measurement(&mut self, measurement: &Measurement) -> Result<()> {
        let (subject, text, html) = create_body(measurement, &measurement.sensor.e_mail_subject.as_ref().unwrap(),
                                                self.text_template, self.html_template)?;
        let email = EmailBuilder::new()
            .to(&measurement.sensor.e_mail_addr.as_ref().unwrap()[..])
            .from(self.from_addr)
            .subject(&subject)
            .alternative(&html, &text)
            .build()?;
        self.send(email)
    }

    fn send(&mut self, email: Email) -> Result<()> {
        match self.transport {
            Transport::File(ref mut file) => file.send(email).map(|_| ()).map_err(|e| e.into()),
            Transport::Smtp(ref mut smtp) => smtp.send(email).map(|_| ()).map_err(|e| e.into()),
            Transport::Stub(ref mut stub) => stub.send(email).map(|_| ()).map_err(|e| e.into()),
        }
    }
}

fn create_body(measurement: &Measurement, subject_template: &str, text_template: &str, html_template: &str) -> Result<(String, String, String)> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("data_value", Box::new(render_data_value));
    let subject = handlebars.template_render(subject_template, measurement)?;
    let text = handlebars.template_render(text_template, measurement)?;
    let html = handlebars.template_render(html_template, measurement)?;

    Ok((subject, text, html))
}

fn render_data_value(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> ::std::result::Result<(), RenderError> {
    let param = h.param(0).unwrap();
    let k_v = param.value().as_object().unwrap();
    let kind = k_v.keys().next().unwrap();
    let value = k_v.values().next().unwrap().as_f64().unwrap();

    let f = format!("{} = {}", kind, value);
    rc.writer.write(f.as_bytes())?;

    Ok(())
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
            id: "123456789".to_string(),
            ui_uri: "http://localhost".to_string(),
            data_uri: "http://localhost".to_string(),
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
        let mut mailer = Mailer {
            transport: Transport::Stub(StubEmailTransport),
            from_addr: "sender@example.com",
            text_template: "{{ sensor.name }}",
            html_template: "{{ sensor.name }}"
        };

        let res = mailer.mail_measurement(&measurement);

        assert!(res.is_ok());
    }
}
