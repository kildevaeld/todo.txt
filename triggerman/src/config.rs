use std::path::PathBuf;

use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use trigger::{BoxTask, Engine, fs_notify::FsNotifyTrigger, manuel::ManuelTrigger};
use vaerdi::Value;

#[typetag::serde(tag = "type")]
pub trait TriggerConfig {
    fn apply(&self, engine: &mut Engine, task: BoxTask<Value>);
}

#[derive(Serialize, Deserialize)]
pub struct NotifyConfig {
    pub paths: Vec<PathBuf>,
    pub recursive: bool,
}

#[typetag::serde]
impl TriggerConfig for NotifyConfig {
    fn apply(&self, engine: &mut Engine, task: BoxTask<Value>) {
        let trigger = FsNotifyTrigger {
            paths: self.paths.clone(),
            recursive: self.recursive,
        };

        engine.add_trigger(trigger, move |event| {
            let task = task.clone();
            async move {
                let value = vaerdi::ser::to_value(event).expect("serialize");
                task.call(value).await
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct ManuelConfig {
    pub name: String,
}

#[typetag::serde]
impl TriggerConfig for ManuelConfig {
    fn apply(&self, engine: &mut Engine, task: BoxTask<Value>) {
        let trigger = ManuelTrigger {
            name: self.name.clone(),
        };

        engine.add_trigger(trigger, move |event| {
            let task = task.clone();
            async move {
                let value = vaerdi::ser::to_value(event).expect("serialize");
                task.call(value).await
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct TaskConfig {
    pub trigger: Box<dyn TriggerConfig>,
    pub work_dir: Option<PathBuf>,
    pub task: RelativePathBuf,
}
