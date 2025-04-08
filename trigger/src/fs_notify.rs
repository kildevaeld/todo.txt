use std::{collections::HashSet, path::PathBuf, time::Duration};

use futures_core::{future::BoxFuture, stream::BoxStream};
use notify_debouncer_full::notify::{self, RecursiveMode};
use tracing::{debug, warn};

use crate::{
    Trigger, TriggerBackend, Worker,
    abort_controller::AbortController,
    backend::{BoxTask, box_task},
};

pub use notify_debouncer_full::notify::{Event, EventKind};

struct FsNotifyTask {
    paths: Vec<PathBuf>,
    recursive: bool,
    work: BoxTask<Event>,
}

fn should_trigger(search_paths: &[PathBuf], event_paths: &[PathBuf], recursive: bool) -> bool {
    for event_path in event_paths {
        if search_paths.contains(event_path) {
            return true;
        }

        if recursive {
            for search_path in search_paths {
                if event_path.starts_with(&search_path) {
                    return true;
                }
            }
        }
    }

    false
}

#[derive(Default)]
pub struct FsNotify {
    tasks: Vec<FsNotifyTask>,
}

impl TriggerBackend for FsNotify {
    type Trigger = FsNotifyTrigger;

    type Stream<'a>
        = BoxStream<'a, Result<Self::Work, Self::Error>>
    where
        Self: 'a;

    type Work = FsNotifyWorker;

    type Arg = Event;

    type Error = notify_debouncer_full::notify::Error;

    fn clear(&mut self) {
        todo!()
    }

    fn add_trigger<W: crate::Task<Self::Arg> + 'static>(
        &mut self,
        trigger: Self::Trigger,
        task: W,
    ) -> Result<(), Self::Error> {
        let paths = trigger
            .paths
            .into_iter()
            .map(|m| m.canonicalize())
            .collect::<Result<_, _>>()?;

        self.tasks.push(FsNotifyTask {
            paths,
            recursive: trigger.recursive,
            work: box_task(task).into(),
        });

        Ok(())
    }

    fn run<'a>(&'a mut self, abort: AbortController) -> Self::Stream<'a> {
        let stream = async_stream::stream! {

            let (sx, mut rx) = tokio::sync::mpsc::channel(10);

            let mut instance = notify_debouncer_full::new_debouncer(Duration::from_secs(2), None,move |event| {
                match event {
                    Ok(ret) => {
                        sx.blocking_send(ret).ok();
                    }
                    Err(err) => {
                        warn!(error = ?err, "Notify error");

                    }
                }
            })?;

            let search_paths: HashSet<_> = self.tasks.iter().map(|m| m.paths.iter().map(|path| (path.clone(), m.recursive))).flatten().collect();

            let instance = tokio::task::spawn_blocking(move || {
                for (path, recursive) in search_paths {
                    debug!(path = ?path, recursive = recursive, "Add watch path");
                    instance.watch(&path, if recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    })?;
                }

               notify::Result::Ok(instance)
            }).await.expect("blocking spawn")?;

            if abort.is_aborted() {
                return;
            }

            loop {
                tokio::select! {
                    next = rx.recv() => {
                        let Some(next) = next else {
                            break
                        };

                        for event in next {
                            for task in &self.tasks {
                                if !should_trigger(&task.paths, &event.paths, task.recursive) {
                                    debug!("not in search path");
                                    continue;
                                }
                                yield Ok(FsNotifyWorker {
                                    work: task.work.clone(),
                                    event:event.event.clone()
                                })
                            }
                        }
                    }
                    _ = abort.wait() => {
                        tokio::task::spawn_blocking(move || {
                            instance.stop();
                        }).await.ok();
                        break
                    }
                };

            }

        };

        Box::pin(stream)
    }
}

pub struct FsNotifyTrigger {
    pub paths: Vec<PathBuf>,
    pub recursive: bool,
}

impl Trigger for FsNotifyTrigger {
    type Backend = FsNotify;
}

pub struct FsNotifyWorker {
    work: BoxTask<Event>,
    event: Event,
}

impl Worker for FsNotifyWorker {
    type Future = BoxFuture<'static, ()>;

    fn call(self) -> Self::Future {
        Box::pin(async move { self.work.call(self.event).await })
    }
}
