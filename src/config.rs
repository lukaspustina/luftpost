use std::fs::File;
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

#[derive(Debug, Deserialize)]
#[serde(tag = "condition")]
#[derive(PartialOrd, PartialEq, Eq)]
#[derive(Clone, Copy)]
pub enum EmailCondition {
    Okay,
    Threshold,
    NoData,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub threshold_pm10: Option<f32>,
    pub threshold_pm2: Option<f32>,
    pub e_mail_addr: Option<String>,
    pub e_mail_subject: Option<String>,
    #[serde(default = "Vec::new")]
    pub e_mail_condition: Vec<EmailCondition>,
}

#[derive(Debug, Deserialize)]
pub struct Sensor {
    pub name: String,
    pub uri: String,
    pub threshold_pm10: Option<f32>,
    pub threshold_pm2: Option<f32>,
    pub e_mail_addr: Option<String>,
    pub e_mail_subject: Option<String>,
    #[serde(default = "Vec::new")]
    pub e_mail_condition: Vec<EmailCondition>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub defaults: Defaults,
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
        let config: Config = toml::from_str(&content)?;

        let config = Config::set_defaults(config);

        Ok(config)
    }

    fn set_defaults(config: Config) -> Config {
        let threshold_pm10 = config.defaults.threshold_pm10.or(Some(50.0));
        let threshold_pm2 = config.defaults.threshold_pm2.or(Some(50.0));
        let e_mail_addr = config.defaults.e_mail_addr;
        let e_mail_subject = config.defaults.e_mail_subject;
        let e_mail_condition = if config.defaults.e_mail_condition.len() > 0 {
            config.defaults.e_mail_condition
        } else {
            vec![EmailCondition::Threshold]
        };

        let sensors = config
            .sensors
            .into_iter()
            .map(|s| {
                let s_threshold_pm10 = s.threshold_pm10.or(threshold_pm10);
                let s_threshold_pm2 = s.threshold_pm2.or(threshold_pm2);
                let s_e_mail_addr = s.e_mail_addr.or(e_mail_addr.clone());
                let s_e_mail_subject = s.e_mail_subject.or(e_mail_subject.clone());
                let s_e_mail_condition = if s.e_mail_condition.len() > 0 {
                    s.e_mail_condition
                } else {
                    e_mail_condition.clone()
                };
                Sensor {
                    threshold_pm10: s_threshold_pm10,
                    threshold_pm2: s_threshold_pm2,
                    e_mail_addr: s_e_mail_addr,
                    e_mail_subject: s_e_mail_subject,
                    e_mail_condition: s_e_mail_condition,
                    ..s
                }
            })
            .collect();

        let defaults = Defaults {
            threshold_pm10: threshold_pm10,
            threshold_pm2: threshold_pm2,
            e_mail_addr: e_mail_addr,
            e_mail_subject: e_mail_subject,
            e_mail_condition: e_mail_condition,
            ..config.defaults
        };
        Config {
            defaults: defaults,
            sensors: sensors,
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
uri = "http://feinstaub/data.json"
"#;

        let config = Config::parse_toml(config_str).unwrap();

        assert_eq!(config.sensors.len(), 1);
    }

    #[test]
    pub fn config_from_max_str_okay() -> () {
        let config_str = r#"[defaults]
threshold_pm10 = 10.0
threshold_pm2 = 10.0
e_mail_addr = "test@example.com"
e_mail_subject = "PM alarm"
[[defaults.e_mail_condition]]
condition = 'Okay'
[[defaults.e_mail_condition]]
condition = 'NoData'

[[sensors]]
name = "Min"
uri = "http://feinstaub/data.json"

[[sensors]]
name = "Max"
uri = "http://feinstaub/data.json"
threshold_pm10 = 20.0
threshold_pm2 = 20.0
e_mail_addr = "another_test@example.com"
e_mail_subject = "Feinstaubalarm"
[[sensors.e_mail_condition]]
condition = 'Threshold'
"#;

        let config = Config::parse_toml(config_str).unwrap();

        assert_eq!(config.defaults.threshold_pm10, Some(10.0));
        assert_eq!(config.defaults.threshold_pm2, Some(10.0));
        assert_eq!(
            config.defaults.e_mail_addr,
            Some("test@example.com".to_string())
        );
        assert_eq!(config.defaults.e_mail_subject, Some("PM alarm".to_string()));
        assert_eq!(
            config.defaults.e_mail_condition,
            vec![EmailCondition::Okay, EmailCondition::NoData]
        );

        assert_eq!(config.sensors.len(), 2);
        let s1 = &config.sensors[0];
        assert_eq!(s1.threshold_pm10, Some(10.0));
        assert_eq!(s1.threshold_pm2, Some(10.0));
        assert_eq!(s1.e_mail_addr, Some("test@example.com".to_string()));
        assert_eq!(s1.e_mail_subject, Some("PM alarm".to_string()));
        assert_eq!(
            s1.e_mail_condition,
            vec![EmailCondition::Okay, EmailCondition::NoData]
        );

        let s2 = &config.sensors[1];
        assert_eq!(s2.threshold_pm10, Some(20.0));
        assert_eq!(s2.threshold_pm2, Some(20.0));
        assert_eq!(s2.e_mail_addr, Some("another_test@example.com".to_string()));
        assert_eq!(s2.e_mail_subject, Some("Feinstaubalarm".to_string()));
        assert_eq!(s2.e_mail_condition, vec![EmailCondition::Threshold]);
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
        let path = Path::new("tests/example_config.toml");

        let config = Config::from_file(&path).unwrap();

        assert_eq!(config.sensors.len(), 1);
    }

    #[test]
    pub fn from_file_not_exists() -> () {
        let path = Path::new("tests/does_not_exist.toml");

        let config = Config::from_file(path.into());

        match config {
            Err(Error(ErrorKind::CouldNotRead(_), _)) => assert!(true),
            _ => assert!(false),
        }
    }
}