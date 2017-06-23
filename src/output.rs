use measurements::{Measurement, ValueType};
use std::io::Write;
use tabwriter::TabWriter;

pub fn output(measurements: &[Measurement]) -> () {
    let mut tw = TabWriter::new(vec![]);
    for ref m in measurements {
        let _ =
            writeln!(
            &mut tw,
            "{}\t({})\tPM 10: {}\tPM 2.5: {}\t Temperature: {}\t Humidity: {}\t Samples: {}\t Min. micro: {}\t Max. micro: {}\t Wifi signal: {}",
            m.sensor.name,
            m.software_version,
            m.data_values.get(&ValueType::SDS_P1).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::SDS_P2).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::TEMPERATURE).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::HUMIDITY).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::SAMPLES).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::MIN_MICRO).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::MAX_MICRO).unwrap_or(&-1.0f32),
            m.data_values.get(&ValueType::SIGNAL).unwrap_or(&-1.0f32),
        );
    }
    tw.flush().unwrap();
    let out_str = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    println!("{}", out_str);
}
