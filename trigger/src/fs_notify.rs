use std::{collections::HashSet, path::PathBuf, sync::Arc};

use futures::{StreamExt, future::BoxFuture, stream::BoxStream};
use notify::{Event, RecursiveMode, Watcher};

use crate::{
    Trigger, TriggerBackend, Worker,
    backend::{BoxTask, BoxWorker, box_task},
};

struct FsNotifyTask {
    paths: Vec<PathBuf>,
    work: Arc<BoxTask<Event>>,
}

pub struct FsNotify {
    tasks: Vec<FsNotifyTask>,
}

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
        self.tasks.push(FsNotifyTask {
            paths: trigger.paths,
            work: box_task(task).into(),
        });
    }

    fn run<'a>(&'a mut self) -> Self::Stream<'a> {
        async_stream::stream! {

            let (sx, mut rx) = tokio::sync::mpsc::channel(10);

            let mut instance = notify::recommended_watcher(move |event| {
                match event 
                sx.blocking_send(event);
            }).unwrap();

            let search_paths: HashSet<_> = self.tasks.iter().map(|m| m.paths.iter()).flatten().cloned().collect();

            let instance = tokio::task::spawn_blocking(move || {
                for path in search_paths {
                    instance.watch(&path, RecursiveMode::Recursive);
                }

                instance
            }).await.unwrap();

            while let Some(next) = rx.recv().await {
                for task in &self.tasks {
                    yield FsNotifyWorker {
                        work: task.task.clone(),
                        event: next
                    }
                }
            }



        }
        .boxed();

        todo!()
    }
}

pub struct FsNotifyTrigger {
    paths: Vec<PathBuf>,
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
