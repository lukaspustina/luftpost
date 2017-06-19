extern crate luftpost;
extern crate tokio_core;

use luftpost::sensors::{create_client, read_measurement};
use luftpost::measurements::ValueType;

use tokio_core::reactor::Core;

#[test]
#[ignore]
fn read_measurement_local() -> () {
    let mut core = Core::new().unwrap();

    let uri = "http://feinstaub/data.json".parse().unwrap();
    let request = create_client(&mut core).get(uri);
    let work = read_measurement(request);
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
