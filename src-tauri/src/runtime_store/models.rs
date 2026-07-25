use crate::runtime_store::error::ContentOperationError;
use uuid::{Uuid, Version};

pub(crate) const MAX_TITLE_BYTES: usize = 512;
pub(crate) const MAX_MESSAGE_CONTENT_BYTES: usize = 262_144;
pub(crate) const MAX_TASK_KIND_BYTES: usize = 64;
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

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "user" => Ok(Self::User),
            "local_runtime" => Ok(Self::LocalRuntime),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

pub(crate) type AuditActor = ContentActor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuditEventType {
    ConversationCreated,
    ConversationDeleted,
    MessageAppended,
    TaskCreated,
    TaskRecorded,
    TaskDeleted,
    RuntimeContentDeleted,
    ExportCompleted,
    StorageRecoveryRequired,
}

impl AuditEventType {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ConversationCreated => "conversation.created",
            Self::ConversationDeleted => "conversation.deleted",
            Self::MessageAppended => "message.appended",
            Self::TaskCreated => "task.created",
            Self::TaskRecorded => "task.recorded",
            Self::TaskDeleted => "task.deleted",
            Self::RuntimeContentDeleted => "runtime.content_deleted",
            Self::ExportCompleted => "export.completed",
            Self::StorageRecoveryRequired => "storage.recovery_required",
        }
    }

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "conversation.created" => Ok(Self::ConversationCreated),
            "conversation.deleted" => Ok(Self::ConversationDeleted),
            "message.appended" => Ok(Self::MessageAppended),
            "task.created" => Ok(Self::TaskCreated),
            "task.recorded" => Ok(Self::TaskRecorded),
            "task.deleted" => Ok(Self::TaskDeleted),
            "runtime.content_deleted" => Ok(Self::RuntimeContentDeleted),
            "export.completed" => Ok(Self::ExportCompleted),
            "storage.recovery_required" => Ok(Self::StorageRecoveryRequired),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuditSubjectType {
    Conversation,
    Message,
    Task,
    Runtime,
    Export,
    Storage,
}

impl AuditSubjectType {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Message => "message",
            Self::Task => "task",
            Self::Runtime => "runtime",
            Self::Export => "export",
            Self::Storage => "storage",
        }
    }

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "conversation" => Ok(Self::Conversation),
            "message" => Ok(Self::Message),
            "task" => Ok(Self::Task),
            "runtime" => Ok(Self::Runtime),
            "export" => Ok(Self::Export),
            "storage" => Ok(Self::Storage),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuditOutcome {
    Success,
    Denied,
    Failed,
}

impl AuditOutcome {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "success" => Ok(Self::Success),
            "denied" => Ok(Self::Denied),
            "failed" => Ok(Self::Failed),
            _ => Err(ContentOperationError::integrity_failure()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuditReasonCode {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InertTaskKind(String);

impl InertTaskKind {
    pub(crate) fn new(value: String) -> Result<Self, ContentOperationError> {
        let bytes = value.as_bytes();
        if bytes.is_empty() || bytes.len() > MAX_TASK_KIND_BYTES || !bytes[0].is_ascii_lowercase() {
            return Err(ContentOperationError::invalid_input());
        }
        for (index, byte) in bytes.iter().copied().enumerate().skip(1) {
            if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
                continue;
            }
            if matches!(byte, b'.' | b'_' | b'-')
                && index + 1 < bytes.len()
                && (bytes[index - 1].is_ascii_lowercase() || bytes[index - 1].is_ascii_digit())
                && (bytes[index + 1].is_ascii_lowercase() || bytes[index + 1].is_ascii_digit())
            {
                continue;
            }
            return Err(ContentOperationError::invalid_input());
        }
        Ok(Self(value))
    }

    pub(super) fn from_database(value: String) -> Result<Self, ContentOperationError> {
        Self::new(value).map_err(|_| ContentOperationError::integrity_failure())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InertTaskState {
    Created,
}

impl InertTaskState {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
        }
    }

    pub(super) fn from_database(value: &str) -> Result<Self, ContentOperationError> {
        match value {
            "created" => Ok(Self::Created),
            _ => Err(ContentOperationError::integrity_failure()),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordInertTaskRequest {
    pub(crate) operation_id: String,
    pub(crate) actor: ContentActor,
    pub(crate) conversation_id: Option<String>,
    pub(crate) task_kind: InertTaskKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GetTaskRequest {
    pub(crate) task_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListTasksRequest {
    pub(crate) limit: u32,
    pub(crate) cursor: Option<TaskCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskRecord {
    pub(crate) id: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) task_kind: InertTaskKind,
    pub(crate) state: InertTaskState,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
    pub(crate) revision: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskCursor {
    pub(crate) updated_at_ms: i64,
    pub(crate) id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskPage {
    pub(crate) items: Vec<TaskRecord>,
    pub(crate) next_cursor: Option<TaskCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GetAuditEventRequest {
    pub(crate) event_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListAuditEventsRequest {
    pub(crate) limit: u32,
    pub(crate) after_sequence_no: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditEventRecord {
    pub(crate) sequence_no: i64,
    pub(crate) event_id: String,
    pub(crate) event_type: AuditEventType,
    pub(crate) actor: AuditActor,
    pub(crate) subject_type: AuditSubjectType,
    pub(crate) subject_id: Option<String>,
    pub(crate) outcome: AuditOutcome,
    pub(crate) reason_code: Option<AuditReasonCode>,
    pub(crate) correlation_id: Option<String>,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditCursor {
    pub(crate) after_sequence_no: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditPage {
    pub(crate) items: Vec<AuditEventRecord>,
    pub(crate) next_cursor: Option<AuditCursor>,
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

impl RecordInertTaskRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.operation_id)?;
        if let Some(conversation_id) = &self.conversation_id {
            validate_uuid_v4(conversation_id)?;
        }
        Ok(())
    }
}

impl GetTaskRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.task_id)
    }
}

impl ListTasksRequest {
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

impl GetAuditEventRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
        validate_uuid_v4(&self.event_id)
    }
}

impl ListAuditEventsRequest {
    pub(super) fn validate(&self) -> Result<(), ContentOperationError> {
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
