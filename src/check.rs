use measurement::{Measurement, Value};

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckedMeasurement {
    pub measurement: Measurement,
    pub has_violations: bool,
    pub violations: Vec<Value>,
}

pub fn check_measurement(measurement: Measurement) -> CheckedMeasurement {
    let violations: Vec<_> = measurement
        .data_values
        .clone()
        .into_iter()
        .map(|value| match value {
            // Unwraps are safe because they are checked during config parsing
            Value::SDS_P1(v) if v > measurement.sensor.threshold_pm10.unwrap() => Some(value),
            Value::SDS_P2(v) if v > measurement.sensor.threshold_pm2.unwrap() => Some(value),
            _ => None,
        })
        .flat_map(|v| v)
        .collect();

    CheckedMeasurement { measurement: measurement, has_violations: !violations.is_empty(), violations: violations }
}

#[cfg(test)]
mod test {
    use super::*;
    use measurement::{Measurement, Value};
    use sensor::Sensor;

    #[test]
    fn check_measurement_okay() -> () {
        let sensor = Sensor {
            name: "A Sensor".to_string(),
            id: "123456789".to_string(),
            ui_uri: "http://localhost".to_string(),
            data_uri: "http://localhost".to_string(),
            threshold_pm10: Some(10.0),
            threshold_pm2: Some(2.0),
            notification_condition: None,
        };
        let mut data_values = Vec::new();
        data_values.push(Value::SDS_P1(17.87f32));
        data_values.push(Value::SDS_P2(3.17f32));
        let measurement = Measurement {
            sensor: sensor,
            software_version: "NRZ-2017-089".to_string(),
            data_values: data_values,
        };

        let res = check_measurement(measurement);

        assert_eq!(res.violations.len(), 2);
        assert!(res.has_violations);

    }
}
