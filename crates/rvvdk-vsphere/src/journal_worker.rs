//! One blocking owner, one bounded command slot, no network or exported file handles.
use crate::{
    contract::{ArtifactId, SourceSelection},
    ownership::{JobStore, OwnershipError, RecoveryReport},
};
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroizing;

mod payload;
pub(crate) use payload::{CHUNK_BYTES, Expected, Progress};

type Result<T> = std::result::Result<T, OwnershipError>;
pub(crate) enum Command {
    Prepare,
    Acquire,
    Held(Zeroizing<String>),
    Abort,
    Aborted,
    PayloadOpen(u64),
    PayloadWrite(Vec<u8>),
    PayloadSeal(Expected),
    Transferred,
    Complete,
    Completed,
}
struct Request {
    command: Command,
    reply: oneshot::Sender<Result<()>>,
}
#[derive(Clone)]
pub(crate) struct Client(mpsc::Sender<Request>);
impl Client {
    pub(crate) async fn apply(&self, command: Command) -> Result<()> {
        let (reply, received) = oneshot::channel();
        self.0
            .send(Request { command, reply })
            .await
            .map_err(|_| OwnershipError::Uncertain)?;
        received.await.map_err(|_| OwnershipError::Uncertain)?
    }
}
pub(crate) struct Outcome {
    pub(crate) recovery: Result<RecoveryReport>,
    pub(crate) progress: Progress,
    pub(crate) payload_error: Option<OwnershipError>,
}
pub(crate) struct Worker {
    pub(crate) client: Client,
    pub(crate) ready: oneshot::Receiver<Result<()>>,
    join: tokio::task::JoinHandle<Outcome>,
}
impl Worker {
    pub(crate) fn start(store: JobStore, artifact: ArtifactId, source: SourceSelection) -> Self {
        Self::start_hook(store, artifact, source, |_| Ok(()))
    }
    // The hook is statically a no-op in production. Tests inject worker delays/errors.
    pub(crate) fn start_hook(
        mut store: JobStore,
        artifact: ArtifactId,
        source: SourceSelection,
        mut hook: impl FnMut(&Command) -> Result<()> + Send + 'static,
    ) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Request>(1);
        let (ready, received) = oneshot::channel();
        let join = tokio::task::spawn_blocking(move || {
            let mut job = match store.create(artifact, &source) {
                Ok(job) => job,
                Err(error) => {
                    let _ = ready.send(Err(error));
                    return Outcome {
                        recovery: Err(error),
                        progress: Progress::default(),
                        payload_error: None,
                    };
                }
            };
            let mut payload = payload::Payload::default();
            let mut payload_error = None;
            // A closed receiver cannot authorize any subsequent resource operation.
            if ready.send(Ok(())).is_ok() {
                while let Some(request) = receiver.blocking_recv() {
                    let is_payload = matches!(
                        request.command,
                        Command::PayloadOpen(_)
                            | Command::PayloadWrite(_)
                            | Command::PayloadSeal(_)
                    );
                    let result = hook(&request.command).and_then(|()| match request.command {
                        Command::Prepare => job.prepare_stage(),
                        Command::Acquire => job.begin_acquire(),
                        Command::Held(reference) => job.acknowledge_lease(&reference),
                        Command::Abort => {
                            payload.close();
                            job.begin_abort()
                        }
                        Command::Aborted => job.acknowledge_abort(),
                        Command::PayloadOpen(limit) => payload.open(&job, limit),
                        Command::PayloadWrite(data) => payload.write(&data),
                        Command::PayloadSeal(expected) => payload.seal(&job, &expected),
                        Command::Transferred
                            if payload.progress.verified && payload_error.is_none() =>
                        {
                            job.validate_stage()?;
                            job.transfer_complete()
                        }
                        Command::Complete
                            if payload.progress.verified && payload_error.is_none() =>
                        {
                            job.validate_stage()?;
                            job.begin_complete()
                        }
                        Command::Completed => job.acknowledge_complete(),
                        _ => Err(OwnershipError::Transition),
                    });
                    let failed = result.is_err();
                    if is_payload && let Err(error) = result {
                        payload_error.get_or_insert(error);
                        payload.close();
                    }
                    let delivered = request.reply.send(result).is_ok();
                    // Dropping a data-write waiter cannot drop its owner. Retain
                    // the serial worker so the next Abort drains it before RPC.
                    // Journal failures/dropped journal acknowledgments still stop.
                    if !is_payload && (failed || !delivered) {
                        break;
                    }
                }
            }
            payload.close();
            // No writers escape this owner. Finish the accepted command, then
            // release its handles before assessment and releasing the store lock.
            drop(job);
            Outcome {
                recovery: store.recover(artifact, &source),
                progress: payload.progress,
                payload_error,
            }
        });
        Self {
            client: Client(sender),
            ready: received,
            join,
        }
    }
    pub(crate) async fn finish(self) -> Result<RecoveryReport> {
        self.finish_payload().await?.recovery
    }
    pub(crate) async fn finish_payload(self) -> Result<Outcome> {
        drop(self.client);
        self.join.await.map_err(|_| OwnershipError::Uncertain)
    }
}

#[cfg(test)]
mod tests;
