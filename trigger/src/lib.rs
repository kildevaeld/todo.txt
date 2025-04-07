mod backend;
mod engine;
mod error;
pub mod manuel;

pub use futures_core::{future::BoxFuture, stream::BoxStream};

#[cfg(feature = "notify")]
pub mod fs_notify;

pub use self::{
    backend::{BoxTask, Task, Trigger, TriggerBackend, Worker, box_task},
    engine::Engine,
};
