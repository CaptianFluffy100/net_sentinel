use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isp {
    pub id: i64,
    pub name: String,
    pub ip: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIsp {
    pub name: String,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Website {
    pub id: i64,
    pub url: String,
    pub direct_connect: bool,
    pub direct_connect_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebsite {
    pub url: String,
    pub direct_connect: bool,
    pub direct_connect_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Udp,
    Tcp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServer {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: Protocol,
    pub timeout_ms: u64,
    pub pseudo_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameServer {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: Protocol,
    pub timeout_ms: u64,
    pub pseudo_code: String,
}

#[derive(Debug, Serialize)]
pub struct GameServerTestResult {
    pub success: bool,
    pub response_time_ms: u64,
    pub raw_response: Option<String>,
    pub parsed_values: serde_json::Value,
    pub error: Option<GameServerError>,
    #[serde(default)]
    pub output_labels_success: Vec<String>,
    #[serde(default)]
    pub output_labels_error: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GameServerError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub line: Option<usize>,
}
