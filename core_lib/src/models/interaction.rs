use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, types::Json};
use time::OffsetDateTime;

/// Defines the specific type of UI interaction required from the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "kebab-case")]
pub enum InteractionType {
    /// User needs to select one option from a provided list.
    SelectFromList,
    /// User needs to confirm or reject a destructive/important action.
    ConfirmAction,
    /// User needs to manually type and submit text.
    InputText,
}

/// Represents a paused state in the pipeline where a plugin requires human
/// intervention.
///
/// When a plugin encounters an ambiguous situation, it creates a `PendingInteraction`.
/// The coordinator then halts the track's processing until an external service
/// resolves this interaction.
#[derive(Debug, Clone, FromRow)]
pub struct PendingInteraction {
    /// The unique identifier of the interaction request in the database.
    pub id: i64,

    /// The ID of the track that is currently paused.
    /// Enforced as UNIQUE in the database to prevent multiple concurrent
    /// requests for a single track.
    pub track_id: i64,

    /// The name of the plugin that requested the interaction.
    pub plugin_name: String,

    /// The category of the UI element that should be presented to the user.
    pub interaction_type: InteractionType,

    /// A flexible JSON payload containing the specific data for the interaction.
    pub payload: Json<serde_json::Value>,

    /// The exact time when the interaction request was created.
    pub created_at: OffsetDateTime,
}
