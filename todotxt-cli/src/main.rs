use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Arg, ArgAction, ArgMatches};
use color_eyre::{eyre::eyre, owo_colors::OwoColorize};
use directories::ProjectDirs;
use inquire::{Text, autocompletion::Replacement};
use todotxt::{Collection, Todo, parser::parse};

mod editor;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let matches = clap::Command::new("Todo.txt")
        .arg(Arg::new("file").short('f'))
        .subcommand(clap::Command::new("new").alias("n").arg(Arg::new("todo")))
        .subcommand(
            clap::Command::new("list")
                .alias("l")
                .arg(Arg::new("context").short('c'))
                .arg(Arg::new("project").short('p'))
                .arg(Arg::new("all").short('a').action(ArgAction::SetTrue)),
        )
        .subcommand(clap::Command::new("edit").alias("e"))
        .get_matches();

    let file_path = if let Some(file) = matches.get_one::<String>("file") {
        PathBuf::from(file)
    } else {
        initialize()?.data_local_dir().join("todo.txt")
    };

    let mut collection = if let Ok(file) = fs::OpenOptions::new().read(true).open(&file_path) {
        Collection::open_reader(file).map_err(|err| eyre!("{err}"))?
    } else {
        Collection::default()
    };

    match matches.subcommand() {
        Some(("new", new_args)) => {
            create_todo(&file_path, &mut collection, new_args)?;
        }
        Some(("list", list_args)) => {
            list_todos(&mut collection, list_args)?;
        }
        Some(("edit", list_args)) => {
            edit_todos(&mut collection, &file_path, list_args)?;
        }
        _ => {}
    };

    Ok(())
}

fn initialize() -> color_eyre::Result<ProjectDirs> {
    let dirs = directories::ProjectDirs::from("com", "Softshag", "Todo.txt")
        .ok_or(eyre!("Could not create directories"))?;

    fs::create_dir_all(dirs.data_local_dir()).ok();

    Ok(dirs)
}

fn create_todo(
    file_path: &Path,
    collection: &mut Collection,
    args: &ArgMatches,
) -> color_eyre::Result<()> {
    let input = if let Some(todo) = args.get_one::<String>("todo") {
        todo.trim().to_string()
    } else {
        let input = Text::new(">")
            .with_autocomplete(AutoCompleter {
                projects: collection
                    .projects()
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect(),
                contexts: collection
                    .contexts()
                    .into_iter()
                    .map(|m| m.to_string())
                    .collect(),
            })
            .prompt()?;

        input.trim().to_string()
    };

    let mut todo = Todo::from(parse(&input)?)?;
    todo.created = Some(chrono::Local::now().date_naive());

    collection.create_todo(todo);

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;

    collection
        .write_writer(&mut file)
        .map_err(|err| eyre!("{err}"))?;

    Ok(())
}

#[derive(Clone)]
struct AutoCompleter {
    projects: Vec<String>,
    contexts: Vec<String>,
}

impl inquire::Autocomplete for AutoCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        if input.ends_with("+") {
            return Ok(self.projects.clone());
        }

        Ok(vec![])
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(Replacement::None)
    }
}

fn list_todos(collection: &mut Collection, args: &ArgMatches) -> color_eyre::Result<()> {
    if collection.is_empty() {
        println!("You have no todos yet...");
        return Ok(());
    }

    let project = args.get_one::<String>("project");

    let all = args.get_flag("all");

    println!("{}", "Todos".underline().bold());
    for todo in collection
        .iter()
        .filter(|m| {
            if let Some(p) = &project {
                m.projects.contains(*p)
            } else {
                true
            }
        })
        .filter(|m| all || !m.done)
    {
        println!("{}", todo);
    }

    Ok(())
}

fn edit_todos(
    collection: &mut Collection,
    file_path: &Path,
    args: &ArgMatches,
) -> color_eyre::Result<()> {
    editor::run(collection)?;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;

    collection
        .write_writer(&mut file)
        .map_err(|err| eyre!("{err}"))?;

    Ok(())
}
