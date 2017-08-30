use sensor::SensorId;
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

error_chain! {
    errors {
        FailedToLoadState(sensor: String, state_dir: String) {
            description("failed to load state")
            display("failed to load state for sensor '{}' from state directory '{}'", sensor, state_dir)
        }
        FailedToSaveState(sensor: String, state_dir: String) {
            description("failed to save state")
            display("failed to save state of sensor '{}' to state directory '{}'", sensor, state_dir)
        }
    }
    foreign_links {
        Io(::std::io::Error);
        JsonError(::serde_json::Error);
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub enum AlarmState {
    Normal,
    ThresholdExceeded,
}

#[derive(Debug, Deserialize, Serialize)]
#[derive(PartialEq)]
pub struct SensorState {
    pub sensor_id: SensorId,
    pub alarm_state: AlarmState,
}

impl SensorState {
    pub fn load<P: AsRef<Path>>(sensor_id: &SensorId, state_dir: P) -> Result<SensorState> {
        let fp = create_filepath(sensor_id, state_dir.as_ref());
        load_from_file(fp).chain_err(|| ErrorKind::FailedToLoadState(sensor_id.clone(), state_dir.as_ref().to_string_lossy().to_string()))
    }

    pub fn save<P: AsRef<Path>>(&self, state_dir: P) -> Result<()> {
        let fp = create_filepath(&self.sensor_id, state_dir.as_ref());
        save_state_to_file(self, fp).chain_err(|| ErrorKind::FailedToSaveState(self.sensor_id.clone(), state_dir.as_ref().to_string_lossy().to_string()))
    }
}

fn create_filepath<P: AsRef<Path>>(sensor_id: &SensorId, state_dir: P) -> PathBuf {
    let mut pb = state_dir.as_ref().to_path_buf();
    let filename = format!("{}.json", sensor_id);
    pb.push(filename);

    pb
}

fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<SensorState> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let state = serde_json::from_str(&content)?;

    Ok(state)
}

fn save_state_to_file<P: AsRef<Path>>(state: &SensorState, file_path: P) -> Result<()> {
    let content = serde_json::to_string(state) ?;

    let mut file = File::create(file_path) ?;
    file.write_all(content.as_bytes()) ?;

    Ok(())
}


    #[cfg(test)]
mod test {
    use super::*;
    use mktemp::Temp;

    #[test]
    pub fn save_state_to_file_ok() -> () {
        let sensor_state = SensorState { sensor_id: "123456789".to_string() , alarm_state: AlarmState::Normal };

        let file = Temp::new_file().unwrap().to_path_buf();
        let res = save_state_to_file(&sensor_state, file);
        assert!(res.is_ok());
    }

    #[test]
    pub fn load_from_file_ok() -> () {
        let expected_sensor_state = SensorState { sensor_id: "123456789".to_string() , alarm_state: AlarmState::ThresholdExceeded};
        let file = Temp::new_file().unwrap().to_path_buf();
        let res = save_state_to_file(&expected_sensor_state,&file);
        assert!(res.is_ok());

        let res = load_from_file(&file);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected_sensor_state);
    }
}