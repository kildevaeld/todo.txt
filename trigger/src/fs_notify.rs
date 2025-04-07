use std::{path::PathBuf, sync::Arc, time::Duration};

use futures::{StreamExt, future::BoxFuture, stream::BoxStream};
use notify_debouncer_full::{notify::Event, DebouncedEvent};

use crate::{
    Trigger, TriggerBackend, Worker,
    backend::{BoxTask, BoxWorker},
};

pub struct FsNotify {

}

impl TriggerBackend for FsNotify {
    type Trigger = FsNotifyTrigger;

    type Stream<'a>
        = BoxStream<'a, Self::Work>
    where
        Self: 'a;

    type Work = FsNotifyWorker;

    type Arg = Event;

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
        async_stream::stream! {

            let (sx, mut rx) = tokio::sync::mpsc::channel(10);

            let instance = notify_debouncer_full::new_debouncer(Duration::from_secs(1), None, move |event| {
                sx.blocking_send(event);
            });

            

            while let Some(next) = rx.recv().await {
               if let Ok(ret) = next {
                for event in ret {
                    
                }
               }
            }



        }
        .boxed();

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
    work: Arc<BoxTask<Event>>,
    event: Event,
}

impl Worker for FsNotifyWorker {
    type Future = BoxFuture<'static, ()>;

    fn call(self) -> Self::Future {
        Box::pin(async move { self.work.call(self.event).await })
    }
}
