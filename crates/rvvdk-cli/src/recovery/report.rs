use super::*;
use rvvdk_vsphere::ownership::{
    PublicationAcknowledgmentRecoveryReport, PublicationAttestationReport, PublicationRecovery,
    ReconciliationReport, RecoveryReport,
};
use serde::Serialize;
use serde_json::{Value, json};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Assessed,
    Completed,
    Failed,
    Invalid,
    Uncertain,
    Cancelled,
    Output,
}
impl Status {
    pub fn code(self) -> i32 {
        match self {
            Self::Assessed | Self::Completed => 0,
            Self::Failed | Self::Output => 1,
            Self::Invalid => 2,
            Self::Uncertain => 3,
            Self::Cancelled => 130,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Assessed => "assessed",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Invalid => "invalid_request",
            Self::Uncertain => "uncertain",
            Self::Cancelled => "cancelled",
            Self::Output => "output_failed",
        }
    }
    fn message(self) -> &'static str {
        match self {
            Self::Assessed | Self::Completed => "local recovery completed",
            Self::Failed => "local recovery failed; inspect retained resources",
            Self::Invalid => "invalid local recovery request; use rvddk recover --help",
            Self::Uncertain => {
                "local outcome is uncertain; inspect retained resources without automatic retry"
            }
            Self::Cancelled => "local recovery cancelled; inspect reported effects",
            Self::Output => "report delivery failed; do not automatically repeat the operation",
        }
    }
}
#[derive(Serialize)]
pub(super) struct Envelope {
    schema_version: u32,
    command: &'static str,
    operation: Option<&'static str>,
    pub status: &'static str,
    pub operation_started: bool,
    pub operation_succeeded: bool,
    assessment_only: bool,
    pub data: Value,
    error: Value,
}
impl Envelope {
    pub fn new(op: Option<Operation>) -> Self {
        Self {
            schema_version: 1,
            command: "recover",
            operation: op.map(Operation::name),
            status: "failed",
            operation_started: false,
            operation_succeeded: false,
            assessment_only: op == Some(Operation::Assess),
            data: Value::Null,
            error: Value::Null,
        }
    }
    pub fn finish(&mut self, s: Status) {
        self.status = s.name();
        self.error = if s.code() == 0 {
            Value::Null
        } else {
            json!({"code":s.name(),"message":s.message()})
        };
    }
    pub fn emit(
        mut self,
        s: Status,
        json_mode: bool,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> i32 {
        self.finish(s);
        let write = |stream: &mut dyn Write| -> std::io::Result<()> {
            if json_mode {
                serde_json::to_writer(&mut *stream, &self)?;
            } else {
                writeln!(
                    stream,
                    "Local recovery: {} ({})",
                    self.status,
                    self.operation.unwrap_or("unspecified")
                )?;
                serde_json::to_writer_pretty(&mut *stream, &self)?;
            }
            writeln!(stream)?;
            stream.flush()
        };
        let result = if s.code() == 0 {
            write(out)
        } else {
            write(err)
        };
        if result.is_ok() {
            return s.code();
        }
        let prior_status = self.status;
        let prior_error = self.error.clone();
        self.finish(Status::Output);
        let fallback = json!({"schema_version":1,"command":"recover","operation":self.operation,"status":self.status,"operation_started":self.operation_started,"operation_succeeded":self.operation_succeeded,"assessment_only":self.assessment_only,"data":self.data,"error":self.error,"operation_status":prior_status,"operation_error":prior_error});
        // Best effort only; neither stream's delivery can be guaranteed.
        let _ = serde_json::to_writer(&mut *err, &fallback);
        let _ = writeln!(err);
        let _ = err.flush();
        1
    }
}
pub(super) fn error_status(e: OwnershipError) -> Status {
    match e {
        OwnershipError::Cancelled => Status::Cancelled,
        OwnershipError::Uncertain => Status::Uncertain,
        _ => Status::Failed,
    }
}
pub(super) fn errors(es: impl IntoIterator<Item = OwnershipError>) -> Option<Status> {
    es.into_iter().map(error_status).max_by_key(|s| match s {
        Status::Uncertain => 3,
        Status::Cancelled => 1,
        _ => 2,
    })
}
fn publication(r: &PublicationRecovery) -> Value {
    json!({"output":{"state":r.output.state,"sequence":r.output.sequence,"pending_transaction":r.output.pending_transaction},"staged":r.staged,"published":r.published})
}
pub(super) fn source(r: &RecoveryReport) -> Value {
    json!({"state":r.state,"sequence":r.sequence,"action":r.action,"pending_transaction":r.pending_transaction})
}
pub(super) fn observation(r: Option<&PublicationRecovery>) -> Value {
    r.map(publication).unwrap_or(Value::Null)
}
pub(super) trait Native {
    fn success(&self) -> bool;
    fn primary(&self) -> Option<OwnershipError>;
    fn assessment(&self) -> Option<OwnershipError>;
    fn recovery(&self) -> Option<&PublicationRecovery>;
    fn data(&self) -> Value;
}
macro_rules! native {
    ($t:ty,$($field:ident),+)=>{
        impl Native for $t {
            fn success(&self)->bool {self.is_success()}
            fn primary(&self)->Option<OwnershipError>{self.primary_error}
            fn assessment(&self)->Option<OwnershipError>{self.assessment_error}
            fn recovery(&self)->Option<&PublicationRecovery>{self.recovery.as_ref()}
            fn data(&self)->Value {json!({$(stringify!($field):self.$field,)+"primary_error":self.primary_error,"assessment_error":self.assessment_error,"recovery":observation(self.recovery.as_ref())})}
        }
    }
}
native!(
    ReconciliationReport,
    origin,
    logical_bytes_verified,
    content_verified,
    bundle_synced,
    intent_synced,
    rename_observed,
    directories_synced,
    acknowledgment_durable
);
native!(
    PublicationAttestationReport,
    logical_bytes_verified,
    content_verified,
    bundle_synced,
    directories_synced,
    published_record_synced,
    bindings_rechecked
);
native!(
    PublicationAcknowledgmentRecoveryReport,
    logical_bytes_verified,
    content_verified,
    bundle_synced,
    intent_synced,
    directories_synced,
    transaction_synced,
    transaction_promoted,
    acknowledgment_durable,
    bindings_rechecked
);
pub(super) fn complete(r: impl Native, e: &mut Envelope) -> Status {
    e.operation_succeeded = r.success();
    e.data = r.data();
    if r.success() {
        Status::Completed
    } else if r.primary() == Some(OwnershipError::Uncertain)
        || r.assessment().is_some()
        || r.recovery().is_none()
        || r.primary().is_none()
    {
        Status::Uncertain
    } else {
        error_status(r.primary().unwrap())
    }
}
#[cfg(test)]
mod tests;
