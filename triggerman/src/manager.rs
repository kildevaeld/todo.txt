use std::path::PathBuf;

use klaver::worker::Worker;
use rquickjs::{CatchResultExt, Function, Module, Object};
use toback::Toback;
use trigger::{AbortController, Engine, box_task, fs_notify::FsNotify, manuel::Manuel};

use crate::{bindings::QuickTask, config::TaskConfig};

const CONFIG_FILE: &'static str = "config.json";

pub struct Manager {
    path: PathBuf,
}

impl Manager {
    pub fn open(path: PathBuf) -> Manager {
        Manager { path }
    }

    pub async fn run(&self, abort: AbortController) {
        let mut engine = Engine::default();

        let (manuel, provider) = Manuel::new();

        engine.add_backend(manuel);
        engine.add_backend(FsNotify::default());

        let mut readdir = tokio::fs::read_dir(&self.path).await.unwrap();

        let toback = Toback::<TaskConfig>::new();

        while let Some(next) = readdir.next_entry().await.unwrap() {
            let path = next.path().join(CONFIG_FILE);
            if !path.try_exists().unwrap() {
                continue;
            }

            let config_content = tokio::fs::read(&path).await.unwrap();
            let config = toback.load(&config_content, "json").unwrap();

            create_task(&mut engine, next.path(), config).await;
        }

        engine.run(Some(abort)).await;
    }
}

async fn create_task(engine: &mut Engine, path: PathBuf, config: TaskConfig) {
    let env = klaver::Options::default()
        .search_path(&path)
        .build_environ();

    let vm = klaver::worker::Worker::new(env, None, None).await.unwrap();

    let script_path = config.task.clone();
    klaver::async_with!(vm => |ctx| {
        let module = Module::import(&ctx, script_path.as_str()).catch(&ctx)?.into_future::<Object>().await.catch(&ctx)?;

        let default = module.get::<_,Function>("default").catch(&ctx)?;

        ctx.globals().set("__$handler", default)?;


        Ok(())
    })
    .await.unwrap();

    let task = box_task(QuickTask { vm });

    config.trigger.apply(engine, task);
}
