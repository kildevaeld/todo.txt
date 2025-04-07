use std::{path::Path, time::Duration};

use futures::{SinkExt, future::BoxFuture};
use notify_debouncer_full::notify::Event;
use trigger::{
    Engine, Task,
    fs_notify::{FsNotify, FsNotifyTrigger},
    manuel::{Manuel, ManuelTrigger},
};

struct Test;

impl Task<()> for Test {
    type Future<'a>
        = BoxFuture<'a, ()>
    where
        Self: 'a,
        String: 'a;

    fn call<'a>(&'a self, input: ()) -> Self::Future<'a> {
        Box::pin(async move {
            println!("Work triggerd");
        })
    }
}

struct Test2(&'static str);

impl Task<Event> for Test2 {
    type Future<'a>
        = BoxFuture<'a, ()>
    where
        Self: 'a,
        String: 'a;

    fn call<'a>(&'a self, input: Event) -> Self::Future<'a> {
        Box::pin(async move {
            println!("{}: Path {:?}", self.0, input);
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut engine = Engine::default();

    let (manuel, mut trigger) = Manuel::new();

    engine.add_backend(manuel);
    engine.add_backend(FsNotify::default());

    engine.add_trigger(
        ManuelTrigger {
            name: "import".to_string(),
        },
        Test,
    );

    engine.add_trigger(
        FsNotifyTrigger {
            paths: vec![Path::new("trigger").canonicalize().unwrap()],
            recursive: true,
        },
        Test2("Src"),
    );

    engine.add_trigger(
        FsNotifyTrigger {
            paths: vec![Path::new("todo.txt").canonicalize().unwrap()],
            recursive: false,
        },
        Test2("Todo"),
    );

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        trigger.send("import".to_string()).await.unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        trigger.send("import".to_string()).await.unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    engine.run().await;
}
