use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(crate) struct StartNotify;

#[derive(Debug, Serialize, Deserialize)]
struct NotificationOptions {
    title: String,
    message: String,
    /// Timeout in milliseconds.
    timeout: Option<u64>,
    level: Option<String>,
}
