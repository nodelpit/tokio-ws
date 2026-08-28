use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { salon: String },
    Leave { salon: String },
    SendMessage { salon: String, text: String },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Message { salon: String, text: String },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_join_message() {
        let json = r#"{
            "type": "join",
            "salon": "general"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::Join { salon } if salon == "general"
        ))
    }

    #[test]
    fn serialize_message() {
        let message = ServerMessage::Message {
            salon: "general".into(),
            text: "hi world".into(),
        };

        let json_string = serde_json::to_string(&message).unwrap();

        let json_value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

        assert_eq!(
            json_value,
            serde_json::json!({
                "type": "message",
                "salon": "general",
                "text": "hi world",
            })
        );
    }
}
