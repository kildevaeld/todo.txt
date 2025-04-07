use std::path::PathBuf;

use toback::Toback;
use trigger::{Engine, fs_notify::FsNotify, manuel::Manuel};

use crate::config::TaskConfig;

const CONFIG_FILE: &'static str = "config.json";

pub struct Manager {
    path: PathBuf,
}

impl Manager {
    pub fn open(&self, path: PathBuf) -> Manager {
        Manager { path }
    }

    pub async fn run(&self) {
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
        }
    }
}
