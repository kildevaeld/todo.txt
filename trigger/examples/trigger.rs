use std::time::Duration;

use futures::{SinkExt, future::BoxFuture};
use trigger::{
    Engine, Task,
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut engine = Engine::default();

    let (manuel, mut trigger) = Manuel::new();

    engine.add_backend(manuel);

    engine.add_trigger(
        ManuelTrigger {
            name: "import".to_string(),
        },
        Test,
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
