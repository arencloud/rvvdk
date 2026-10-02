//! One blocking owner, one bounded command slot, no network or exported file handles.
use crate::{
    contract::{ArtifactId, SourceSelection},
    ownership::{JobStore, OwnershipError, RecoveryReport},
};
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, OwnershipError>;
pub(crate) enum Command {
    Prepare,
    Acquire,
    Held(Zeroizing<String>),
    Abort,
    Aborted,
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
pub(crate) struct Worker {
    pub(crate) client: Client,
    pub(crate) ready: oneshot::Receiver<Result<()>>,
    join: tokio::task::JoinHandle<Result<RecoveryReport>>,
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
                    return Err(error);
                }
            };
            // A closed receiver cannot authorize any subsequent resource operation.
            if ready.send(Ok(())).is_ok() {
                while let Some(request) = receiver.blocking_recv() {
                    let result = hook(&request.command).and_then(|()| match request.command {
                        Command::Prepare => job.prepare_stage(),
                        Command::Acquire => job.begin_acquire(),
                        Command::Held(reference) => job.acknowledge_lease(&reference),
                        Command::Abort => job.begin_abort(),
                        Command::Aborted => job.acknowledge_abort(),
                    });
                    let failed = result.is_err();
                    let delivered = request.reply.send(result).is_ok();
                    if failed || !delivered {
                        break;
                    }
                }
            }
            // No writers escape this owner. Finish the accepted command, then
            // release its handles before assessment and releasing the store lock.
            drop(job);
            store.recover(artifact, &source)
        });
        Self {
            client: Client(sender),
            ready: received,
            join,
        }
    }
    pub(crate) async fn finish(self) -> Result<RecoveryReport> {
        drop(self.client);
        self.join.await.map_err(|_| OwnershipError::Uncertain)?
    }
}

#[cfg(test)]
mod tests;
