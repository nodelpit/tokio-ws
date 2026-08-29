use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { salon: String },
    Leave { salon: String },
    SendMessage { salon: String, text: String },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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
    fn deserialize_leave_message() {
        let json = r#"{
            "type": "leave",
            "salon": "general"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::Leave { salon } if salon == "general"
        ))
    }

    #[test]
    fn deserialize_send_message() {
        let json = r#"{
            "type": "send_message",
            "salon": "general",
            "text": "hi world"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::SendMessage { salon, text } if salon == "general" && text == "hi world"
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

    #[test]
    fn roundtrip_message() {
        let message = ServerMessage::Message {
            salon: "general".into(),
            text: "hi world".into(),
        };

        let json_string = serde_json::to_string(&message).unwrap();

        let json_to_rust: ServerMessage = serde_json::from_str(&json_string).unwrap();

        assert_eq!(message, json_to_rust);
    }

    #[test]
    fn reject_malformed_message() {
        let invalid_json = "not json";
        assert!(serde_json::from_str::<ClientMessage>(invalid_json).is_err());

        let unknown_type = r#"{
            "type": "unknown",
            "salon": "general"
        }"#;
        assert!(serde_json::from_str::<ClientMessage>(unknown_type).is_err());

        let missing_field = r#"{
            "type": "join"
        }"#;
        assert!(serde_json::from_str::<ClientMessage>(missing_field).is_err());
    }
}
