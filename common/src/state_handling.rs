use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct ActivationState {
    pub irrigator: Option<bool>,
    pub heater: Option<bool>,
    pub lighting: Option<bool>,
    pub uv: Option<bool>,
    pub shading: Option<bool>,
}

impl ActivationState {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<ActivationState> for HashMap<String, bool> {
    //Map only existing values
    fn from(value: ActivationState) -> Self {
        let mut hm = HashMap::new();
        if let Some(irrigator) = value.irrigator {
            hm.insert("irrigator".to_string(), irrigator);
        }
        if let Some(heater) = value.heater {
            hm.insert("heater".to_string(), heater);
        }
        if let Some(lighting) = value.lighting {
            hm.insert("lighting".to_string(), lighting);
        }
        if let Some(uv) = value.uv {
            hm.insert("uv".to_string(), uv);
        }
        if let Some(shading) = value.shading {
            hm.insert("shading".to_string(), shading);
        }
        hm
    }
}
