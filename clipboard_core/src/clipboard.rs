use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "Type")]
pub enum ClipboardData {
    Text {
        #[serde(rename = "Clipboard")]
        content: String,
        #[serde(rename = "Html")]
        html: Option<String>,
        #[serde(rename = "File")]
        file: Option<String>,
        #[serde(rename = "Device", alias = "device", default)]
        device: Option<String>,
    },
    Image {
        #[serde(rename = "Clipboard")]
        hash: Option<String>,
        #[serde(rename = "File")]
        filename: String,
        #[serde(rename = "Device", alias = "device", default)]
        device: Option<String>,
    },
    File {
        #[serde(rename = "Clipboard")]
        hash: Option<String>,
        #[serde(rename = "File")]
        filename: String,
        #[serde(rename = "Device", alias = "device", default)]
        device: Option<String>,
    },
}

impl ClipboardData {
    pub fn new_text(content: String) -> Self {
        ClipboardData::Text {
            content,
            html: None,
            file: None,
            device: None,
        }
    }

    // TODO: Hash calculation for images and files
}
