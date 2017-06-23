use measurement::{Measurement, Value};
use std::io::Write;
use tabwriter::TabWriter;

pub fn print_measurements(measurements: &[Measurement]) -> () {
    let mut tw = TabWriter::new(vec![]);
    for m in measurements {
        let values_str = m.data_values
            .iter()
            .map(|value| match *value {
                Value::SDS_P1(v) => format!("PM 10: {}", v),
                Value::SDS_P2(v) => format!("PM 2.5: {}", v),
                Value::TEMPERATURE(v) => format!("Temperature: {}", v),
                Value::HUMIDITY(v) => format!("Humidity: {}", v),
                Value::SAMPLES(v) => format!("Samples: {}", v),
                Value::MIN_MICRO(v) => format!("Min. micro: {}", v),
                Value::MAX_MICRO(v) => format!("Max. micro: {}", v),
                Value::SIGNAL(v) => format!("Wifi signal: {}", v),
                Value::UNKNOWN(ref s) => format!("Unknown value: {}", s),
            })
            .collect::<Vec<_>>()
            .join("\t");
        let _ = writeln!(
            &mut tw,
            "{}\t({})\t{}",
            m.sensor.name,
            m.software_version,
            values_str
        );
    }
    tw.flush().unwrap();
    let out_str = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    println!("{}", out_str);
}
