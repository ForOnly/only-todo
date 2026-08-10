use serde::{Deserialize, Serialize};

pub mod priority;
pub mod reminder;
pub mod status;
pub mod todo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub notification_enabled: bool,
    pub list_default_sort: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsDto {
    #[serde(default)]
    pub notification_enabled: Option<bool>,
    #[serde(default)]
    pub list_default_sort: Option<String>,
}
