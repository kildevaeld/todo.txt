use std::{collections::BTreeMap, sync::Arc};

use futures::{StreamExt, future::BoxFuture, stream::BoxStream};

use crate::{
    Trigger, TriggerBackend, Worker,
    backend::{BoxTask, box_task},
};

pub struct Manuel {
    triggers: BTreeMap<String, Arc<BoxTask<()>>>,
    rx: futures::channel::mpsc::Receiver<String>,
}

impl Manuel {
    pub fn new() -> (Manuel, futures::channel::mpsc::Sender<String>) {
        let (sx, rx) = futures::channel::mpsc::channel(1);
        (
            Manuel {
                triggers: Default::default(),
                rx,
            },
            sx,
        )
    }
}

impl TriggerBackend for Manuel {
    type Trigger = ManuelTrigger;

    type Stream<'a>
        = BoxStream<'a, Self::Work>
    where
        Self: 'a;

    type Work = ManuelWorker;

    type Arg = ();

    fn clear(&mut self) {
        todo!()
    }

    fn add_trigger<W: crate::Task<Self::Arg> + 'static>(
        &mut self,
        trigger: Self::Trigger,
        task: W,
    ) {
        if self.triggers.contains_key(&trigger.name) {
            panic!("Already contains key");
        }
        self.triggers.insert(trigger.name, Arc::new(box_task(task)));
    }

    fn run<'a>(&'a mut self) -> Self::Stream<'a> {
        let stream = async_stream::stream! {


          while let Some(next) = self.rx.next().await {
            let Some(found) = self.triggers.get(&next) else {
                continue;
            };

            yield ManuelWorker {
              task: found.clone()
            };
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
