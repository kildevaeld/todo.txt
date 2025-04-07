use futures_core::{future::BoxFuture, stream::BoxStream};
use std::{collections::BTreeMap, sync::Arc};

use crate::{
    Trigger, TriggerBackend, Worker,
    backend::{BoxTask, box_task},
    error::Error,
};

pub struct Manuel {
    triggers: BTreeMap<String, Arc<BoxTask<()>>>,
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl Manuel {
    pub fn new() -> (Manuel, ManuelSender) {
        let (sx, rx) = tokio::sync::mpsc::channel(1);
        (
            Manuel {
                triggers: Default::default(),
                rx,
            },
            ManuelSender { sx },
        )
    }
}

impl TriggerBackend for Manuel {
    type Trigger = ManuelTrigger;

    type Stream<'a>
        = BoxStream<'a, Result<Self::Work, Self::Error>>
    where
        Self: 'a;

    type Work = ManuelWorker;

    type Arg = ();

    type Error = Error;

    fn clear(&mut self) {
        todo!()
    }

    fn add_trigger<W: crate::Task<Self::Arg> + 'static>(
        &mut self,
        trigger: Self::Trigger,
        task: W,
    ) -> Result<(), Self::Error> {
        if self.triggers.contains_key(&trigger.name) {
            return Err(Error::new("Already contains key"));
        }
        self.triggers.insert(trigger.name, Arc::new(box_task(task)));

        Ok(())
    }

    fn run<'a>(&'a mut self) -> Self::Stream<'a> {
        let stream = async_stream::stream! {


          while let Some(next) = self.rx.recv().await {
            let Some(found) = self.triggers.get(&next) else {
                continue;
            };

            yield Ok(ManuelWorker {
              task: found.clone()
            });
          }

        };

        Box::pin(stream)
    }
}

pub struct ManuelTrigger {
    pub name: String,
}

impl Trigger for ManuelTrigger {
    type Backend = Manuel;
}

pub struct ManuelWorker {
    task: Arc<BoxTask<()>>,
}

impl Worker for ManuelWorker {
    type Future = BoxFuture<'static, ()>;

    fn call(self) -> Self::Future {
        Box::pin(async move { self.task.call(()).await })
    }
}

#[derive(Clone)]
pub struct ManuelSender {
    sx: tokio::sync::mpsc::Sender<String>,
}

impl ManuelSender {
    pub async fn trigger(&self, name: impl Into<String>) {
        self.sx.send(name.into()).await.ok();
    }

    pub fn blocking_trigger(&self, name: impl Into<String>) {
        self.sx.blocking_send(name.into()).ok();
    }
}
