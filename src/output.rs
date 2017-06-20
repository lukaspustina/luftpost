use measurements::{Measurement, ValueType};
use std::io::Write;
use tabwriter::TabWriter;

pub fn output(measurements: &[Measurement]) -> () {
    let mut tw = TabWriter::new(vec![]);
    for ref m in measurements {
        let _ =
            writeln!(
            &mut tw,
            "Name is missing\t({})\tSDS_P1: {}",
            m.software_version,
            m.data_values.get(&ValueType::SDS_P1).unwrap_or(&-1.0f32),
        );
    }
    tw.flush().unwrap();
    let out_str = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    println!("{}", out_str);
}
