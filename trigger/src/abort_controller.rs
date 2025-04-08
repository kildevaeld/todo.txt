use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::broadcast::{Sender, channel};

#[derive(Clone)]
pub struct AbortController {
    sx: Sender<()>,
    cancled: Arc<AtomicBool>,
}

impl Default for AbortController {
    fn default() -> Self {
        let (sx, _) = channel(1);
        AbortController {
            sx,
            cancled: Default::default(),
        }
    }
}

impl AbortController {
    pub fn wait(&self) -> impl Future<Output = ()> + Send {
        let mut rx = self.sx.subscribe();
        async move {
            rx.recv().await.ok();
        }
    }

    pub fn trigger(&self) {
        self.cancled.store(true, Ordering::SeqCst);
        self.sx.send(()).ok();
    }

    pub fn is_aborted(&self) -> bool {
        self.cancled.load(Ordering::SeqCst)
    }
}
