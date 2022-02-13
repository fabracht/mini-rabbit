

use actix::Message;
use lapin::options::ExchangeDeclareOptions;
use lapin::ExchangeKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishToTopic {
    pub exchange: String,
    pub routing_key: String,
    pub payload: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConsumeFromExchange {
    pub queue: String,
    pub exchange: String,
    pub connection_name: String,
    pub self_id: Uuid,
    pub room_id: Uuid,
}

#[derive(Debug, Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct CreateExchange {
    pub exchange_name: String,
    pub exchange_options: Option<ExchangeDeclareOptions>,
    pub exchange_kind: ExchangeKind,
}

impl From<Value> for CreateExchange {
    fn from(value: Value) -> Self {
        let exchange_name = value.get("exchange_name")
            .expect("Missing exchange_name value")
            .to_string().splitn(3, '"').collect::<String>();
        let exchange_kind = value.get("exchange_kind").expect("Missing exchange_type").as_str().expect("Can't create string slice from exchange_king");
        let exchange = match exchange_kind {
            "Topic" => ExchangeKind::Topic,
            "Fanout" => ExchangeKind::Fanout,
            "Direct" => ExchangeKind::Direct,
            "Headers" => ExchangeKind::Headers,
            _ => ExchangeKind::Custom(exchange_kind.to_owned())
        };
        let exchange_options_value = value.get("exchange_options").expect("Can't get exchange_options value").to_owned();
        let exchange_options = serde_json::from_value::<ExchangeDeclareOptions>(exchange_options_value);
        if let Ok(options) = exchange_options {
            Self {
                exchange_name,
                exchange_options: Some(options),
                exchange_kind: exchange
            }
        } else {
            Self {
                exchange_name,
                exchange_options: Some(ExchangeDeclareOptions::default()),
                exchange_kind: exchange
            }
        }

    }
}