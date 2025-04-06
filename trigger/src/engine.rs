use core::any::Any;

use crate::backend::{AnyBackend, BackendBox, Task, Trigger, TriggerBackend, Worker};
use futures::StreamExt;

#[derive(Default)]
pub struct Engine {
    backends: Vec<Box<dyn AnyBackend>>,
}

impl Engine {
    pub fn add_backend<T>(&mut self, backend: T)
    where
        T: TriggerBackend + 'static,
        T::Work: Send,
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

    pub async fn run(&mut self) {
        let mut tasks = futures::stream::select_all(self.backends.iter_mut().map(|m| m.run()));

        let concurrency = 10;

        let waitgroup = tokio::sync::Semaphore::new(concurrency);

        while let Some(task) = tasks.next().await {
            let permit = waitgroup.acquire().await.expect("Semaphore open");
            tokio::spawn(async move {
                let _ = permit;
                task.run().await;
            });
        }
    }
}
