use std::{
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
};

use clap::{Arg, ArgAction, ArgMatches, Command};
use color_eyre::{eyre::eyre, owo_colors::OwoColorize};
use directories::ProjectDirs;
use editor2::ListBox;
use inquire::Text;
use projects::Projects;
use todotxt::{Collection, Todo, parser::parse};

mod editor;
mod editor2;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let matches = clap::Command::new("Todo.txt")
        .arg(Arg::new("file").short('f'))
        .subcommand(
            clap::Command::new("new")
                .alias("n")
                .arg(Arg::new("todo").help("Todo in todo.txt format"))
                .about("Create a new todo"),
        )
        .subcommand(
            clap::Command::new("list")
                .alias("l")
                .arg(Arg::new("context").short('c'))
                .arg(Arg::new("project").short('p').help("Filter by project"))
                .arg(
                    Arg::new("all")
                        .short('a')
                        .action(ArgAction::SetTrue)
                        .help("Also list completed todos"),
                )
                .about("List todos"),
        )
        .subcommand(
            clap::Command::new("edit")
                .alias("e")
                .arg(Arg::new("project").required(true)),
        )
        .subcommand(
            Command::new("readme")
                .alias("r")
                .arg(Arg::new("project").required(true)),
        )
        .get_matches();

    let mut projects = Projects::open()?;

    match matches.subcommand() {
        Some(("new", new_args)) => {
            create_todo(&mut projects, new_args)?;
        }
        Some(("list", list_args)) => {
            list_todos(&mut projects, list_args)?;
        }
        Some(("readme", readme_args)) => {
            readme(&mut projects, readme_args)?;
        }
        Some(("edit", list_args)) => {
            edit_todos(&mut projects, list_args)?;
        }
        _ => {}
    };

    Ok(())
}

fn create_todo(projects: &mut Projects, args: &ArgMatches) -> color_eyre::Result<()> {
    let input = if let Some(todo) = args.get_one::<String>("todo") {
        todo.trim().to_string()
    } else {
        let input = Text::new(">")
            .with_autocomplete(AutoCompleter {
                projects: projects.iter().map(|m| m.name().to_string()).collect(),
                contexts: Vec::new(),
            })
            .prompt_skippable()?;

        let Some(input) = input else { return Ok(()) };

        input.trim().to_string()
    };

    let mut todo = Todo::from(parse(&input)?)?;
    todo.created = Some(chrono::Local::now().date_naive());

    if todo.projects.is_empty() {
        eprintln!("No project specified");
        return Ok(());
    }

    for project_name in &todo.projects {
        let project = if let Some(project) = projects.find_mut(&*project_name) {
            project
        } else {
            projects.create(project_name.clone()).unwrap()
        };

        project.todos_mut().create_todo(todo.clone());
    }

    projects.sync().unwrap();

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
        } else if input.ends_with("@") {
            return Ok(self.contexts.clone());
        }

        Ok(vec![])
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(highlighted_suggestion)
    }
}

fn list_todos(projects: &mut Projects, args: &ArgMatches) -> color_eyre::Result<()> {
    if projects.is_empty() {
        println!("You have no projects yet...");
        return Ok(());
    }

    if std::io::stdout().is_terminal() {
        println!("{}", "Projects".underline().bold());
    }
    for project in projects.iter() {
        println!("{}", project.name());
    }

    Ok(())
}

fn readme(projects: &mut Projects, args: &ArgMatches) -> color_eyre::Result<()> {
    let project_name = args.get_one::<String>("project").unwrap();

    let project = if let Some(project) = projects.find_mut(&*project_name) {
        project
    } else {
        projects.create(project_name.clone()).unwrap()
    };

    let out = inquire::Editor::new("Readme")
        .with_file_extension("md")
        .with_predefined_text(&project.description())
        .prompt_skippable()?;

    let Some(out) = out else { return Ok(()) };

    *project.description_mut() = out;

    projects.sync()?;

    Ok(())
}

// fn edit_todos(projects: &mut Projects, args: &ArgMatches) -> color_eyre::Result<()> {
//     let project_name = args.get_one::<String>("project").unwrap();

//     let project = if let Some(project) = projects.find_mut(&*project_name) {
//         project
//     } else {
//         projects.create(project_name.clone()).unwrap()
//     };

//     if !editor::run(project.todos_mut())? {
//         return Ok(());
//     }

//     projects.sync()?;

//     Ok(())
// }

fn edit_todos(projects: &mut Projects, args: &ArgMatches) -> color_eyre::Result<()> {
    for _ in 0..5 {
        println!();
    }

    let (_, current_row) = crossterm::cursor::position()?;
    let start = current_row - 5;

    editor2::run(&mut ListBox::new(
        vec!["Hello".to_string(), "World".to_string()],
        5,
        start,
    ))?;

    Ok(())
}
