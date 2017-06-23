extern crate luftpost;
extern crate tokio_core;

use luftpost::sensors::{Sensor, create_client};
use luftpost::measurements::ValueType;

use tokio_core::reactor::Core;

#[test]
#[ignore]
fn read_measurement_local() -> () {
    let mut core = Core::new().unwrap();

    let uri = "http://feinstaub/data.json".parse().unwrap();
    let response = create_client(&mut core).get(uri);
    let work = Sensor::new("A Sensor", "http://localhost").read_measurement(response);
    let res = core.run(work).unwrap();

    assert!(res.data_values.contains_key(&ValueType::SDS_P1));
    assert!(res.data_values.contains_key(&ValueType::SDS_P2));
    assert!(res.data_values.contains_key(&ValueType::TEMPERATURE));
    assert!(res.data_values.contains_key(&ValueType::HUMIDITY));
    assert!(res.data_values.contains_key(&ValueType::SAMPLES));
    assert!(res.data_values.contains_key(&ValueType::MIN_MICRO));
    assert!(res.data_values.contains_key(&ValueType::MAX_MICRO));
    assert!(res.data_values.contains_key(&ValueType::SIGNAL));
}
