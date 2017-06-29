use sensor::Sensor;
use std::fmt;

error_chain! {
    errors {
        InvalidValueType(type_str: String) {
            description("invalid value type")
                display("invalid value type: '{}'", type_str)
        }
        InvalidValue(value_str: String) {
            description("invalid value")
                display("invalid value: '{}'", value_str)
        }
    }
    links {
        WireMeasurementDecodingFailed(wire::Error, wire::ErrorKind);
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum Value {
    SDS_P1(f32),
    SDS_P2(f32),
    TEMPERATURE(f32),
    HUMIDITY(f32),
    SAMPLES(f32),
    MIN_MICRO(f32),
    MAX_MICRO(f32),
    SIGNAL(f32),
    UNKNOWN(String),
}

impl<'a> From<(&'a str, f32)> for Value {
    fn from(type_tuple: (&'a str, f32)) -> Self {
        let (type_str, value) = type_tuple;
        let upper = &type_str.to_uppercase()[..];
        match upper {
            "SDS_P1" => Value::SDS_P1(value),
            "SDS_P2" => Value::SDS_P2(value),
            "TEMPERATURE" => Value::TEMPERATURE(value),
            "HUMIDITY" => Value::HUMIDITY(value),
            "SAMPLES" => Value::SAMPLES(value),
            "MIN_MICRO" => Value::MIN_MICRO(value),
            "MAX_MICRO" => Value::MAX_MICRO(value),
            "SIGNAL" => Value::SIGNAL(value),
            _ => Value::UNKNOWN(type_str.to_string()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Value::SDS_P1(v) => format!("PM 10 = {}", v),
            Value::SDS_P2(v) => format!("PM 2.5 = {}", v),
            Value::TEMPERATURE(v) => format!("Temperature = {}", v),
            Value::HUMIDITY(v) => format!("Humidity = {}", v),
            Value::SAMPLES(v) => format!("Samples = {}", v),
            Value::MIN_MICRO(v) => format!("Min micro = {}", v),
            Value::MAX_MICRO(v) => format!("Max Micro = {}", v),
            Value::SIGNAL(v) => format!("Wifi signal = {}", v),
            Value::UNKNOWN(ref v) => format!("Unknown = {}", v),
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Measurement {
    pub sensor: Sensor,
    pub software_version: String,
    pub data_values: Vec<Value>,
}

impl Measurement {
    pub fn from_json(sensor: Sensor, json: &str) -> Result<Self> {
        let wire_measurement = wire::decode_json_to_measurement(json)?;
        wire_to_measurement(sensor, wire_measurement)
    }
}

fn wire_to_measurement(sensor: Sensor, wire: wire::Measurement) -> Result<Measurement> {
    let mut data_values = Vec::new();

    for dv in wire.data_values {
        let value_str = dv.value;
        let value = value_str
            .parse::<f32>()
            .chain_err(|| ErrorKind::InvalidValue(value_str.to_string()))?;
        match Value::from((&dv.value_type[..], value)) {
            Value::UNKNOWN(str) => bail!(ErrorKind::InvalidValueType(str)),
            vt => data_values.push(vt),
        }
    }

    Ok(Measurement {
        sensor: sensor,
        software_version: wire.software_version,
        data_values: data_values,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn value_type_from_str_ok() -> () {
        assert_eq!(Value::SDS_P1(10.0), ("SDS_P1", 10.0).into());
        assert_eq!(Value::SDS_P2(11.0), ("SDS_P2", 11.0).into());
        assert_eq!(Value::TEMPERATURE(12.0), ("TEMPERATURE", 12.0).into());
        assert_eq!(Value::HUMIDITY(13.0), ("HUMIDITY", 13.0).into());
        assert_eq!(Value::SAMPLES(14.0), ("SAMPLES", 14.0).into());
        assert_eq!(Value::MIN_MICRO(15.0), ("MIN_MICRO", 15.0).into());
        assert_eq!(Value::MAX_MICRO(16.0), ("MAX_MICRO", 16.0).into());
        assert_eq!(Value::SIGNAL(17.0), ("SIGNAL", 17.0).into());
    }

    #[test]
    fn value_type_from_str_unknown() -> () {
        assert_eq!(
            Value::UNKNOWN("does not exists".to_string()),
            ("does not exists", 10.0).into()
        );
    }

    #[test]
    fn wire_to_measurement_ok() -> () {
        let w_data_values: Vec<wire::DataValue> = vec![
            wire::DataValue {
                value_type: "SDS_P1".to_string(),
                value: "7.87".to_string(),
            },
            wire::DataValue {
                value_type: "SDS_P2".to_string(),
                value: "3.17".to_string(),
            },
            wire::DataValue {
                value_type: "temperature".to_string(),
                value: "18.90".to_string(),
            },
            wire::DataValue {
                value_type: "humidity".to_string(),
                value: "49.10".to_string(),
            },
            wire::DataValue {
                value_type: "samples".to_string(),
                value: "739514".to_string(),
            },
            wire::DataValue {
                value_type: "min_micro".to_string(),
                value: "192".to_string(),
            },
            wire::DataValue {
                value_type: "max_micro".to_string(),
                value: "27599".to_string(),
            },
            wire::DataValue {
                value_type: "signal".to_string(),
                value: "-73".to_string(),
            },
        ];
        let wire = wire::Measurement {
            software_version: "NRZ-2017-089".to_string(),
            data_values: w_data_values,
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
        let expected = Measurement {
            sensor: Sensor::new("A Sensor", "123456789", "http://localhost", "http://localhost"),
            software_version: "NRZ-2017-089".to_string(),
            data_values: data_values,
        };

        let m = wire_to_measurement(Sensor::new("A Sensor", "123456789", "http://localhost", "http://localhost"), wire);

        assert_eq!(m.unwrap(), expected);
    }

    #[test]
    fn wire_to_measurement_unknown_value_type() -> () {
        let w_data_values: Vec<wire::DataValue> = vec![
            wire::DataValue {
                value_type: "this data type does not exists".to_string(),
                value: "7.87".to_string(),
            },
        ];
        let wire = wire::Measurement {
            software_version: "NRZ-2017-089".to_string(),
            data_values: w_data_values,
        };

        let res = wire_to_measurement(Sensor::new("A Sensor", "123456789", "http://localhost", "http://localhost"), wire);

        match res {
            Err(Error(ErrorKind::InvalidValueType(_), _)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn wire_to_measurement_invalid_value() -> () {
        let w_data_values: Vec<wire::DataValue> = vec![
            wire::DataValue {
                value_type: "SDS_P1".to_string(),
                value: "invalid float".to_string(),
            },
        ];
        let wire = wire::Measurement {
            software_version: "NRZ-2017-089".to_string(),
            data_values: w_data_values,
        };

        let res = wire_to_measurement(Sensor::new("A Sensor", "123456789", "http://localhost", "http://localhost"), wire);

        match res {
            Err(Error(ErrorKind::InvalidValue(_), _)) => assert!(true),
            _ => assert!(false),
        }
    }
}

mod wire {
    use serde_json;

    error_chain! {
        errors {
            InvalidJson(json: String) {
                description("invalid measurement json")
                    display("invalid measurement json: '{}'", json)
            }

        }
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Measurement {
        pub software_version: String,
        #[serde(rename(deserialize = "sensordatavalues"))]
        pub data_values: Vec<DataValue>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct DataValue {
        pub value_type: String,
        pub value: String,
    }

    pub fn decode_json_to_measurement<T: Into<String>>(json: T) -> Result<Measurement> {
        let json_string = json.into();
        let m: Measurement = serde_json::from_str(&json_string)
            .chain_err(|| ErrorKind::InvalidJson(json_string))?;

        Ok(m)
    }


    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn decode_json_to_measurement_ok() -> () {
            let json = r#"{"software_version": "NRZ-2017-089", "sensordatavalues":[{"value_type":"SDS_P1","value":"7.87"},{"value_type":"SDS_P2","value":"3.17"},{"value_type":"temperature","value":"18.90"},{"value_type":"humidity","value":"49.10"},{"value_type":"samples","value":"739514"},{"value_type":"min_micro","value":"192"},{"value_type":"max_micro","value":"27599"},{"value_type":"signal","value":"-73"}]}"#;
            let data_values: Vec<DataValue> = vec![
                DataValue {
                    value_type: "SDS_P1".to_string(),
                    value: "7.87".to_string(),
                },
                DataValue {
                    value_type: "SDS_P2".to_string(),
                    value: "3.17".to_string(),
                },
                DataValue {
                    value_type: "temperature".to_string(),
                    value: "18.90".to_string(),
                },
                DataValue {
                    value_type: "humidity".to_string(),
                    value: "49.10".to_string(),
                },
                DataValue {
                    value_type: "samples".to_string(),
                    value: "739514".to_string(),
                },
                DataValue {
                    value_type: "min_micro".to_string(),
                    value: "192".to_string(),
                },
                DataValue {
                    value_type: "max_micro".to_string(),
                    value: "27599".to_string(),
                },
                DataValue {
                    value_type: "signal".to_string(),
                    value: "-73".to_string(),
                },
            ];
            let expected = Measurement {
                software_version: "NRZ-2017-089".to_string(),
                data_values: data_values,
            };

            let res = decode_json_to_measurement(json);

            assert_eq!(expected, res.unwrap());
        }

        #[test]
        fn decode_json_to_measurement_invalid_json() -> () {
            let json = r#"{some invalid stuff"#;

            let res = decode_json_to_measurement(json);

            match res {
                Err(Error(ErrorKind::InvalidJson(_), _)) => assert!(true),
                _ => assert!(false),
            }
        }
    }
}
