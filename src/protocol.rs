use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    Join { room: String },
    Leave { room: String },
    SendMessage { room: String, text: String },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Message { room: String, text: String },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_join_message() {
        let json = r#"{
            "type": "join",
            "room": "general"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::Join { room } if room == "general"
        ))
    }

    #[test]
    fn deserialize_leave_message() {
        let json = r#"{
            "type": "leave",
            "room": "general"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::Leave { room } if room == "general"
        ))
    }

    #[test]
    fn deserialize_send_message() {
        let json = r#"{
            "type": "send_message",
            "room": "general",
            "text": "hi world"
        }"#;

        let message: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(
            message,
            ClientMessage::SendMessage { room, text } if room == "general" && text == "hi world"
        ))
    }

    #[test]
    fn serialize_message() {
        let message = ServerMessage::Message {
            room: "general".into(),
            text: "hi world".into(),
        };

        let json_string = serde_json::to_string(&message).unwrap();

        let json_value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

        assert_eq!(
            json_value,
            serde_json::json!({
                "type": "message",
                "room": "general",
                "text": "hi world",
            })
        );
    }

    #[test]
    fn serialize_error() {
        let message = ServerMessage::Error {
            message: "error message".into(),
        };

        let json_string = serde_json::to_string(&message).unwrap();

        let json_value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

        assert_eq!(
            json_value,
            serde_json::json!({
                "type": "error",
                "message": "error message",
            })
        );
    }

    #[test]
    fn roundtrip_message() {
        let message = ServerMessage::Message {
            room: "general".into(),
            text: "hi world".into(),
        };

        let json_string = serde_json::to_string(&message).unwrap();

        let json_to_rust: ServerMessage = serde_json::from_str(&json_string).unwrap();

        assert_eq!(message, json_to_rust);
    }

    #[test]
    fn reject_invalid_json() {
        let result = serde_json::from_str::<ClientMessage>("not json");
        assert!(result.unwrap_err().is_syntax());
    }

    #[test]
    fn reject_unknown_type() {
        let result =
            serde_json::from_str::<ClientMessage>(r#"{"type":"unknown","room":"general"}"#);
        assert!(result.unwrap_err().is_data());
    }

    #[test]
    fn reject_missing_field() {
        let result = serde_json::from_str::<ClientMessage>(r#"{"type":"join"}"#);
        assert!(result.unwrap_err().is_data());
    }

    #[test]
    fn deny_unknown_fields() {
        let result =
            serde_json::from_str::<ClientMessage>(r#"{"type":"join","room":"general","extra":1}"#);
        assert!(result.unwrap_err().is_data());
    }
}
