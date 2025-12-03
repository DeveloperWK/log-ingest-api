#[derive(Debug, serde::Deserialize)]
pub struct RawLog {
    pub message: String,
    pub level: String,
    pub service: String,
    pub timestamp: Option<i64>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub host: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct IngestLog {
    pub id: uuid::Uuid,
    pub received_at: i64,
    pub client_ip: String,
    // pub api_key: String,
    pub message: String,
    pub level: String,
    pub service: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub host: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
