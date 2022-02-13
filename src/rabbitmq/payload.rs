use serde::{Serialize, Deserialize};
use serde_json::{Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    #[serde(flatten)]
    pub data: Value,
}

impl ToString for Payload {
    fn to_string(&self) -> String {
        serde_json::ser::to_string(self).expect("Error serializing payload")
    }
}

// impl From<Value> for Payload {
//     fn from(value: Value) -> Self {
//         Self {
//             connection_name: value.get("connection_name").expect("Missing connection_name").to_string(),
//             connection_type: value.get("connection_type").expect("Missing connection_type")
//                 .to_string(),
//             message_type: value.get("message_type").expect("Missing connection_name").to_string(),
//             message_frequency: value.get("message_frequency").map(|v| v.to_string()),
//             room: value.get("room").map(|v| v.to_string()),
//         }
//     }
// }