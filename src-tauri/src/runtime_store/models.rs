use crate::runtime_store::error::ContentOperationError;
use uuid::{Uuid, Version};

pub(crate) const MAX_TITLE_BYTES: usize = 512;
pub(crate) const MAX_MESSAGE_CONTENT_BYTES: usize = 262_144;
pub(crate) const MAX_PAGE_SIZE: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContentActor {
    User,
    LocalRuntime,
}

impl ContentActor {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::LocalRuntime => "local_runtime",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationStatus {
    Active,
    Archived,
}

impl ConversationStatus {
    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConversationCursor {
    pub(crate) updated_at_ms: i64,
    pub(crate) id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateConversationRequest {
    pub(crate) operation_id: String,
    pub(crate) actor: ContentActor,
    pub(crate) title: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GetConversationRequest {
    pub(crate) conversation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListConversationsRequest {
    pub(crate) limit: u32,
    pub(crate) cursor: Option<ConversationCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppendMessageRequest {
    pub(crate) operation_id: String,
    pub(crate) actor: ContentActor,
    pub(crate) conversation_id: String,
    pub(crate) role: MessageRole,
    pub(crate) content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListMessagesRequest {
    pub(crate) conversation_id: String,
    pub(crate) limit: u32,
    pub(crate) after_sequence_no: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConversationRecord {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) status: ConversationStatus,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
    pub(crate) next_message_sequence: i64,
    pub(crate) revision: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MessageRecord {
    pub(crate) id: String,
    pub(crate) conversation_id: String,
    pub(crate) sequence_no: i64,
    pub(crate) role: MessageRole,
    pub(crate) content: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConversationPage {
    pub(crate) items: Vec<ConversationRecord>,
    pub(crate) next_cursor: Option<ConversationCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MessagePage {
    pub(crate) items: Vec<MessageRecord>,
    pub(crate) next_after_sequence_no: Option<i64>,
}

impl CreateConversationRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.operation_id)?;
        if self
            .title
            .as_ref()
            .is_some_and(|title| title.len() > MAX_TITLE_BYTES)
        {
            return Err(ContentOperationError::invalid_input());
        }
        Ok(())
    }
}

impl GetConversationRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.conversation_id)
    }
}

impl ListConversationsRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_limit(self.limit)?;
        if let Some(cursor) = &self.cursor {
            validate_uuid_v4(&cursor.id)?;
            if cursor.updated_at_ms < 0 {
                return Err(ContentOperationError::invalid_input());
            }
        }
        Ok(())
    }
}

impl AppendMessageRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.operation_id)?;
        validate_uuid_v4(&self.conversation_id)?;
        if self.content.is_empty() || self.content.len() > MAX_MESSAGE_CONTENT_BYTES {
            return Err(ContentOperationError::invalid_input());
        }
        Ok(())
    }
}

impl ListMessagesRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.conversation_id)?;
        validate_limit(self.limit)?;
        if self
            .after_sequence_no
            .is_some_and(|sequence_no| sequence_no <= 0)
        {
            return Err(ContentOperationError::invalid_input());
        }
        Ok(())
    }
}

pub(super) fn validate_uuid_v4(value: &str) -> Result<(), ContentOperationError> {
    let parsed = Uuid::parse_str(value).map_err(|_| ContentOperationError::invalid_input())?;
    if parsed.get_version() != Some(Version::Random) || parsed.to_string() != value {
        return Err(ContentOperationError::invalid_input());
    }
    Ok(())
}

fn validate_limit(limit: u32) -> Result<(), ContentOperationError> {
    if (1..=MAX_PAGE_SIZE).contains(&limit) {
        Ok(())
    } else {
        Err(ContentOperationError::invalid_input())
    }
}
