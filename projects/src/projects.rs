use std::{
    io,
    path::{Path, PathBuf},
};

use todotxt::{Collection, Todo};

const DESCRIPTION_FILE: &'static str = "README.md";
const TODOTXT_FILE: &'static str = "todo.txt";

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Error {}

pub struct Project {
    name: String,
    description: String,
    todos: Collection,
    dirty: bool,
}

impl Project {
    fn new(name: String) -> Project {
        Project {
            name,
            description: String::default(),
            todos: Collection::default(),
            dirty: false,
        }
    }

    fn open(path: &Path) -> Result<Project, BoxError> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let description = std::fs::read_to_string(path.join(DESCRIPTION_FILE)).unwrap_or_default();
        let todos = if let Ok(file) = std::fs::OpenOptions::new()
            .read(true)
            .open(path.join(TODOTXT_FILE))
        {
            Collection::open_reader(file)?
        } else {
            Collection::default()
        };

        Ok(Project {
            name,
            description,
            todos,
            dirty: false,
        })
    }

    fn write(&self, root: &Path) -> Result<(), BoxError> {
        let project_path = root.join(&self.name);
        std::fs::create_dir_all(&project_path)?;
        std::fs::write(project_path.join(DESCRIPTION_FILE), &self.description)?;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(project_path.join(TODOTXT_FILE))?;

        self.todos.write_writer(&mut file)?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn description_mut(&mut self) -> &mut String {
        self.dirty = true;
        &mut self.description
    }

    pub fn todos(&self) -> &Collection {
        &self.todos
    }

    pub fn todos_mut(&mut self) -> &mut Collection {
        self.dirty = true;
        &mut self.todos
    }
}

pub struct Projects {
    projects: Vec<Project>,
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl Projects {
    pub fn open() -> Result<Projects, BoxError> {
        let dirs = directories::ProjectDirs::from("com", "Softshag", "Projects")
            .expect("Config directory");

        let data_dir = dirs.data_local_dir().to_path_buf();
        let config_dir = dirs.config_local_dir().to_path_buf();
        let mut projects = Vec::default();
        let readdir = std::fs::read_dir(&data_dir)?;

        for entry in readdir {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let project = Project::open(&entry.path())?;
            projects.push(project);
        }

        Ok(Projects {
            projects,
            data_dir,
            config_dir,
        })
    }

    pub fn sync(&self) -> Result<(), BoxError> {
        for project in &self.projects {
            if project.dirty {
                project.write(&self.data_dir)?;
            }
        }
        Ok(())
    }

    pub fn create(&mut self, name: String) -> Result<&mut Project, Error> {
        if self.projects.iter().any(|m| m.name == name) {
            panic!("Project already defined");
        }

        self.projects.push(Project::new(name.clone()));

        self.find_mut(&name).ok_or_else(|| todo!())
    }

    pub fn find(&mut self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|m| m.name.as_str() == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Project> {
        self.projects.iter_mut().find(|m| m.name.as_str() == name)
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Project> {
        self.projects.iter()
    }
}
