extern crate luftpost;
extern crate tokio_core;

use luftpost::{Sensor, create_sensor_reader, print_measurements};

use tokio_core::reactor::Core;

#[test]
#[ignore]
fn read_measurement_local() -> () {
    let mut core = Core::new().unwrap();

    let uri = "http://feinstaub/data.json".parse().unwrap();
    let response = create_sensor_reader(&mut core).get(uri);
    let work = Sensor::new("A Sensor", "http://localhost").read_measurement(response);
    let res = core.run(work).unwrap();

    assert_eq!(res.data_values.len(), 8);
    print_measurements(&vec!(res));
}
