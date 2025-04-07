use core::any::Any;
use std::sync::Arc;

use futures_core::{Stream, future::BoxFuture, stream::BoxStream};
use futures_util::{StreamExt, TryStreamExt};

use crate::error::{BoxError, Error};

pub trait Task<I>: Send + Sync {
    type Future<'a>: Future<Output = ()> + Send
    where
        Self: 'a,
        I: 'a;
    fn call<'a>(&'a self, input: I) -> Self::Future<'a>;
}

pub trait DynTask<I>: Send + Sync {
    fn call<'a>(&'a self, input: I) -> BoxFuture<'a, ()>;
}

pub type BoxTask<I> = Arc<dyn DynTask<I>>;

pub trait Worker {
    type Future: Future<Output = ()>;
    fn call(self) -> Self::Future;
}

pub trait TriggerBackend {
    type Trigger: Trigger<Backend = Self>;
    type Stream<'a>: Stream<Item = Result<Self::Work, Self::Error>>
    where
        Self: 'a;
    type Work: Worker;
    type Arg;
    type Error;

    fn clear(&mut self);
    fn add_trigger<W: Task<Self::Arg> + 'static>(
        &mut self,
        trigger: Self::Trigger,
        task: W,
    ) -> Result<(), Self::Error>;

    fn run<'a>(&'a mut self) -> Self::Stream<'a>;
}

pub trait Trigger {
    type Backend: TriggerBackend<Trigger = Self>;
}

pub trait AnyBackend: Any {
    fn run<'a>(&'a mut self) -> BoxStream<'a, Result<BoxWorker, Error>>;
}

pub type BoxWorker = Box<dyn DynWorker + Send>;

pub trait DynWorker {
    fn run(self: Box<Self>) -> BoxFuture<'static, ()>;
}

struct WorkerBox<T>(T);

impl<T> DynWorker for WorkerBox<T>
where
    T: Worker,
    T::Future: Send + 'static,
{
    fn run(self: Box<Self>) -> BoxFuture<'static, ()> {
        Box::pin(self.0.call())
    }
}

pub struct BackendBox<T>(pub T);

impl<T: 'static> AnyBackend for BackendBox<T>
where
    T: TriggerBackend,
    T::Error: Into<BoxError>,
    T::Work: Send,
    for<'a> T::Stream<'a>: Send,
    <T::Work as Worker>::Future: Send,
{
    fn run<'a>(&'a mut self) -> BoxStream<'a, Result<BoxWorker, Error>> {
        self.0
            .run()
            .map_ok(|worker| Box::new(WorkerBox(worker)) as BoxWorker)
            .map_err(Error::new)
            .boxed()
    }
}

pub fn box_task<T, I>(task: T) -> BoxTask<I>
where
    T: Task<I> + 'static,
    I: 'static,
    for<'a> T::Future<'a>: Send,
{
    Arc::from(TaskBox(task))
}

struct TaskBox<T>(T);

impl<T, I> DynTask<I> for TaskBox<T>
where
    T: Task<I>,
    for<'a> T::Future<'a>: Send,
    I: 'static,
{
    fn call<'a>(&'a self, input: I) -> BoxFuture<'a, ()> {
        Box::pin(self.0.call(input))
    }
}
