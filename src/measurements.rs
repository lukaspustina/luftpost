use std::collections::HashMap;

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

#[derive(Debug, PartialEq)]
pub struct Measurement {
    pub software_version: String,
    pub data_values: HashMap<ValueType, f32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ValueType {
    SDS_P1,
    SDS_P2,
    TEMPERATURE,
    HUMIDITY,
    SAMPLES,
    MIN_MICRO,
    MAX_MICRO,
    SIGNAL,
    UNKNOWN(String),
}

impl<'a> From<&'a str> for ValueType {
    fn from(type_str: &'a str) -> Self {
        let upper = &type_str.to_uppercase()[..];
        match upper {
            "SDS_P1" => ValueType::SDS_P1,
            "SDS_P2" => ValueType::SDS_P2,
            "TEMPERATURE" => ValueType::TEMPERATURE,
            "HUMIDITY" => ValueType::HUMIDITY,
            "SAMPLES" => ValueType::SAMPLES,
            "MIN_MICRO" => ValueType::MIN_MICRO,
            "MAX_MICRO" => ValueType::MAX_MICRO,
            "SIGNAL" => ValueType::SIGNAL,
            _ => ValueType::UNKNOWN(type_str.to_string()),
        }
    }
}

pub fn measurement_from_json(json: &str) -> Result<Measurement> {
    let wire_measurement = wire::decode_json_to_measurement(json)?;
    wire_to_measurement(wire_measurement)
}

fn wire_to_measurement(wire: wire::Measurement) -> Result<Measurement> {
    let mut data_values = HashMap::new();

    for dv in wire.data_values {
        let value_str = dv.value;
        let value = value_str
            .parse::<f32>()
            .chain_err(|| ErrorKind::InvalidValue(value_str.to_string()))?;
        match ValueType::from(&dv.value_type[..]) {
            ValueType::UNKNOWN(str) => bail!(ErrorKind::InvalidValueType(str)),
            vt => {
                data_values.insert(vt, value);
            }
        }
    }

    Ok(Measurement {
        software_version: wire.software_version,
        data_values: data_values,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn value_type_from_str_ok() -> () {
        assert_eq!(ValueType::SDS_P1, "SDS_P1".into());
        assert_eq!(ValueType::SDS_P2, "SDS_P2".into());
        assert_eq!(ValueType::TEMPERATURE, "TEMPERATURE".into());
        assert_eq!(ValueType::HUMIDITY, "HUMIDITY".into());
        assert_eq!(ValueType::SAMPLES, "SAMPLES".into());
        assert_eq!(ValueType::MIN_MICRO, "MIN_MICRO".into());
        assert_eq!(ValueType::MAX_MICRO, "MAX_MICRO".into());
        assert_eq!(ValueType::SIGNAL, "SIGNAL".into());
    }

    #[test]
    fn value_type_from_str_unknown() -> () {
        assert_eq!(
            ValueType::UNKNOWN("does not exists".to_string()),
            "does not exists".into()
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

        let mut data_values = HashMap::new();
        data_values.insert(ValueType::SDS_P1, 7.87f32);
        data_values.insert(ValueType::SDS_P2, 3.17f32);
        data_values.insert(ValueType::TEMPERATURE, 18.90f32);
        data_values.insert(ValueType::HUMIDITY, 49.10f32);
        data_values.insert(ValueType::SAMPLES, 739514f32);
        data_values.insert(ValueType::MIN_MICRO, 192f32);
        data_values.insert(ValueType::MAX_MICRO, 27599f32);
        data_values.insert(ValueType::SIGNAL, -73f32);
        let expected = Measurement {
            software_version: "NRZ-2017-089".to_string(),
            data_values: data_values,
        };

        let m = wire_to_measurement(wire);

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

        let res = wire_to_measurement(wire);

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

        let res = wire_to_measurement(wire);

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
