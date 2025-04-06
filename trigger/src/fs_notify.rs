use std::{path::PathBuf, sync::Arc};

use futures::{future::BoxFuture, stream::BoxStream};

use crate::{
    Trigger, TriggerBackend, Worker,
    backend::{BoxTask, BoxWorker},
};

pub struct FsNotify {}

impl TriggerBackend for FsNotify {
    type Trigger = FsNotifyTrigger;

    type Stream<'a>
        = BoxStream<'a, Self::Work>
    where
        Self: 'a;

    type Work = FsNotifyWorker;

    type Arg = notify::Event;

    fn clear(&mut self) {
        todo!()
    }

    fn add_trigger<W: crate::Task<Self::Arg> + 'static>(
        &mut self,
        trigger: Self::Trigger,
        task: W,
    ) {
        todo!()
    }

    fn run<'a>(&'a mut self) -> Self::Stream<'a> {
        todo!()
    }
}

pub struct FsNotifyTrigger {
    path: Vec<PathBuf>,
}

impl Trigger for FsNotifyTrigger {
    type Backend = FsNotify;
}

pub struct FsNotifyWorker {
    work: Arc<BoxTask<notify::Event>>,
    event: notify::Event,
}

impl Worker for FsNotifyWorker {
    type Future = BoxFuture<'static, ()>;

    fn call(self) -> Self::Future {
        Box::pin(async move { self.work.call(self.event).await })
    }
}
