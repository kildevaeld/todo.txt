use core::any::Any;
use std::time::Duration;

use futures_util::StreamExt;

use crate::{
    abort_controller::AbortController,
    backend::{AnyBackend, BackendBox, Task, Trigger, TriggerBackend, Worker},
    error::BoxError,
};
// use futures::StreamExt;

pub struct Engine {
    backends: Vec<Box<dyn AnyBackend>>,
    concurrency: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            backends: Default::default(),
            concurrency: 10,
        }
    }
}

impl Engine {
    pub fn add_backend<T>(&mut self, backend: T)
    where
        T: TriggerBackend + 'static,
        T::Work: Send,
        T::Error: Into<BoxError>,
        for<'a> T::Stream<'a>: Send,
        <T::Work as Worker>::Future: Send,
    {
        self.backends.push(Box::new(BackendBox(backend)));
    }

    pub fn add_trigger<T, W>(&mut self, trigger: T, task: W)
    where
        T: Trigger,
        T::Backend: 'static,
        W: Task<<T::Backend as TriggerBackend>::Arg> + 'static,
    {
        if let Some(found) = self
            .backends
            .iter_mut()
            .find_map(|m| (&mut **m as &mut dyn Any).downcast_mut::<BackendBox<T::Backend>>())
        {
            found.0.add_trigger(trigger, task);
        } else {
            panic!(
                "Backend {:?} not registerd",
                core::any::type_name::<T::Backend>()
            )
        }
    }

    pub async fn run(&mut self, abort: Option<AbortController>) {
        let abort = abort.unwrap_or_default();
        let mut tasks = futures_util::stream::select_all(
            self.backends.iter_mut().map(|m| m.run(abort.clone())),
        );

        let semaphore = tokio::sync::Semaphore::new(self.concurrency);

        let mut waitgroup = awaitgroup::WaitGroup::new();

        loop {
            if abort.is_aborted() {
                break;
            }

            tokio::select! {
                next = tasks.next() => {
                    let Some(next) = next else {
                        break
                    };
                    let task = match next {
                        Ok(ret) => ret,
                        Err(err) => {
                            println!("Task failed: {}", err);
                            continue;
                        }
                    };

                    let permit = semaphore.acquire().await.expect("Semaphore open");
                    if abort.is_aborted() {
                        break
                    }

                    let worker = waitgroup.worker();

                    tokio::spawn(async move {
                        let _ = permit;
                        task.run().await;
                        worker.done();
                    });
                }
                _ = abort.wait() => {
                    tracing::debug!("Closing");
                    break;
                }
            }
        }

        tokio::time::timeout(Duration::from_secs(5), waitgroup.wait())
            .await
            .ok();

        // while let Some(next) = tasks.next().await {
        //     let task = match next {
        //         Ok(ret) => ret,
        //         Err(err) => {
        //             println!("Task failed: {}", err);
        //             continue;
        //         }
        //     };

        //     let permit = waitgroup.acquire().await.expect("Semaphore open");
        //     tokio::spawn(async move {
        //         let _ = permit;
        //         task.run().await;
        //     });
        // }
    }
}
