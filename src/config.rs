use sensor::Sensor;

use lettre::transport::smtp::authentication;
use serde::de::{self, Deserializer, Visitor};
use std::fs::File;
use std::fmt;
use std::io::Read;
use std::path::Path;
use toml;

error_chain! {
	errors {
	}
	foreign_links {
		CouldNotRead(::std::io::Error);
		CouldNotParse(::toml::de::Error);
	}
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "condition")]
#[derive(PartialOrd, PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum NotificationCondition {
    Always,
    ThresholdExceeded,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub threshold_pm10: Option<f32>,
    pub threshold_pm2: Option<f32>,
    pub notification_condition: Option<NotificationCondition>,
}

#[derive(Debug, Deserialize)]
pub struct Smtp {
    pub sender: String,
    pub receiver: String,
    pub subject: String,
    pub server: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(deserialize_with = "auth_mechanism")]
    pub auth_mechanism: Option<authentication::Mechanism>,
    #[serde(default = "default_template")]
    pub text_template: String,
    #[serde(default = "default_template")]
    pub html_template: String,
}

//fn deserialize_u64_or_empty_string<D>(deserializer: &mut D) -> Result<u64, D::Error> where D: Deserializer
fn auth_mechanism<'de, D>(
    deserializer: D,
) -> ::std::result::Result<Option<authentication::Mechanism>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MechanismVisitor;

    impl<'a> Visitor<'a> for MechanismVisitor {
        type Value = Option<authentication::Mechanism>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string with valid values 'CramMd5' or 'Plain'")
        }

        fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            match &s.to_uppercase()[..] {
                "CRAMMD5" => Ok(Some(authentication::Mechanism::CramMd5)),
                "PLAIN" => Ok(Some(authentication::Mechanism::Plain)),
                _ => Err(de::Error::custom("vaild values are 'CramMd5' and 'Plain'")),
            }
        }
    }

    deserializer.deserialize_string(MechanismVisitor)
}

fn default_template() -> String {
    "{{ sensor.name }}".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub defaults: Defaults,
    pub smtp: Option<Smtp>,
    pub sensors: Vec<Sensor>,
}

impl Config {
    pub fn from_file(file_path: &Path) -> Result<Config> {
        let mut file = File::open(file_path)?;
        let content = Config::read_to_string(&mut file)?;

        Config::parse_toml(&content)
    }

    fn read_to_string(file: &mut File) -> Result<String> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    fn parse_toml(content: &str) -> Result<Config> {
        let config: Config = toml::from_str(content)?;

        let config = Config::set_defaults(config);

        Ok(config)
    }

    fn set_defaults(config: Config) -> Config {
        let threshold_pm10 = config.defaults.threshold_pm10.or(Some(50.0));
        let threshold_pm2 = config.defaults.threshold_pm2.or(Some(50.0));
        let e_mail_condition = config.defaults.notification_condition.or(Some(NotificationCondition::ThresholdExceeded));

        let sensors = config
            .sensors
            .into_iter()
            .map(|s| {
                let s_threshold_pm10 = s.threshold_pm10.or(threshold_pm10);
                let s_threshold_pm2 = s.threshold_pm2.or(threshold_pm2);
                let s_notification_condition = s.notification_condition.or_else(|| e_mail_condition.clone());
                Sensor {
                    threshold_pm10: s_threshold_pm10,
                    threshold_pm2: s_threshold_pm2,
                    notification_condition: s_notification_condition,
                    ..s
                }
            })
            .collect();

        let defaults = Defaults {
            threshold_pm10: threshold_pm10,
            threshold_pm2: threshold_pm2,
            notification_condition: e_mail_condition,
        };
        Config {
            defaults: defaults,
            sensors: sensors,
            ..config
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn config_from_min_str_okay() -> () {
        let config_str = r#"[defaults]

[[sensors]]
name = "Feinstaub"
id = "12345678"
ui_uri = "http://feinstaub"
data_uri = "http://feinstaub/data.json"
"#;

        let config = Config::parse_toml(config_str).unwrap();

        assert_eq!(config.sensors.len(), 1);
    }

    #[test]
    pub fn config_from_max_str_okay() -> () {
        let config_str = r#"[defaults]
threshold_pm10 = 10.0
threshold_pm2 = 10.0
[defaults.notification_condition]
condition = 'Always'

[smtp]
sender = "test@example.com"
subject = "PM alarm from sensor {{ sensor.name }}"
receiver = "test@example.com"
server = "localhost"
port = 25
username = "test"
password = "example"
auth_mechanism = "CramMd5"
text_template = """Hello,

your sensor {{ sensor.name }} just found a measurement exceeding a threashold."""
html_template = """Hello,

your sensor {{ sensor.name }} just found a measurement exceeding a threshold."""

[[sensors]]
name = "Min"
id = "12345678"
ui_uri = "http://feinstaub"
data_uri = "http://feinstaub/data.json"

[[sensors]]
name = "Max"
id = "12345678"
ui_uri = "http://feinstaub"
data_uri = "http://feinstaub/data.json"
threshold_pm10 = 20.0
threshold_pm2 = 20.0
e_mail_addr = "another_test@example.com"
e_mail_subject = "Feinstaubalarm"
[sensors.notification_condition]
condition = 'ThresholdExceeded'
"#;

        let config = Config::parse_toml(config_str).unwrap();

        assert_eq!(config.defaults.threshold_pm10.unwrap(), 10.0);
        assert_eq!(config.defaults.threshold_pm2.unwrap(), 10.0);
        assert_eq!(config.defaults.notification_condition.unwrap(), NotificationCondition::Always);

        assert!(config.smtp.is_some());
        let smtp = config.smtp.unwrap();
        assert_eq!( &smtp.receiver, "test@example.com" );
        assert!(smtp.subject.contains("{{ sensor.name }}"));
        assert_eq!(&smtp.sender, "test@example.com");
        assert_eq!(&smtp.server, "localhost");
        assert_eq!(smtp.port.unwrap(), 25);
        assert_eq!(smtp.username.as_ref().unwrap(), "test");
        assert_eq!(smtp.password.as_ref().unwrap(), "example");
        assert_eq!(
            smtp.auth_mechanism.unwrap(),
            authentication::Mechanism::CramMd5
        );
        assert!(smtp.text_template.contains("{{ sensor.name }}"));
        assert!(smtp.html_template.contains("{{ sensor.name }}"));

        assert_eq!(config.sensors.len(), 2);
        let s1 = &config.sensors[0];
        assert_eq!(s1.threshold_pm10.unwrap(), 10.0);
        assert_eq!(s1.threshold_pm2.unwrap(), 10.0);
        assert_eq!(s1.notification_condition.unwrap(), NotificationCondition::Always);

        let s2 = &config.sensors[1];
        assert_eq!(s2.threshold_pm10.unwrap(), 20.0);
        assert_eq!(s2.threshold_pm2.unwrap(), 20.0);
        assert_eq!(s2.notification_condition.unwrap(), NotificationCondition::ThresholdExceeded);
    }

    #[test]
    pub fn config_from_str_parser_error() -> () {
        let config_str = r#"[defaults"#;

        let config = Config::parse_toml(config_str);

        match config {
            Err(Error(ErrorKind::CouldNotParse(_), _)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    pub fn from_file() -> () {
        let path = Path::new("tests/luftpost.example.conf");

        let config = Config::from_file(&path).unwrap();

        assert_eq!(config.sensors.len(), 1);
    }

    #[test]
    pub fn from_file_not_exists() -> () {
        let path = Path::new("tests/does_not_exist.conf");

        let config = Config::from_file(path.into());

        match config {
            Err(Error(ErrorKind::CouldNotRead(_), _)) => assert!(true),
            _ => assert!(false),
        }
    }
}
