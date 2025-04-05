use core::fmt;
use std::{
    collections::{BTreeSet, HashMap},
    io::BufRead,
};

use chrono::NaiveDate;

use crate::parser::{Item, parse};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Date(NaiveDate),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Date(d) => write!(f, "{d}"),
            Value::Float(i) => write!(f, "{i}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Todo {
    pub description: String,
    pub created: Option<NaiveDate>,
    pub completed: Option<NaiveDate>,
    pub contexts: Vec<String>,
    pub projects: Vec<String>,
    pub done: bool,
    pub values: HashMap<String, Vec<Value>>,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.done {
            write!(f, "X ")?;
        }

        if let Some(completed) = self.completed {
            write!(f, "{} ", completed)?;
        }

        if let Some(created) = self.created {
            write!(f, "{} ", created)?;
        }

        write!(f, "{}", self.description)?;

        for project in &self.projects {
            write!(f, " +{}", project)?;
        }

        for context in &self.contexts {
            write!(f, " @{}", context)?;
        }

        for (k, vals) in &self.values {
            for v in vals {
                write!(f, " {}:{}", k, v)?;
            }
        }

        Ok(())
    }
}

impl Todo {
    pub fn from(todo: crate::parser::Todo<'_>) -> Result<Todo, udled::Error> {
        let mut out = Todo {
            description: todo.description.as_str().into(),
            contexts: Vec::default(),
            projects: Default::default(),
            done: todo.done,
            values: Default::default(),
            completed: None,
            created: None,
        };

        if let Some(created) = todo.created {
            out.created = Some(created.value);
        }

        if let Some(completed) = todo.completed {
            out.completed = Some(completed.value);
        }

        for item in todo.items {
            match item {
                Item::Context(ctx) => {
                    out.contexts.push(ctx.value.to_string());
                }
                Item::KeyVal { key, value } => {
                    let value = match value {
                        crate::parser::Value::Bool(b) => Value::Bool(b.value),
                        crate::parser::Value::Int(b) => Value::Int(b.value as i64),
                        crate::parser::Value::Float(f) => Value::Float(f.value),
                        crate::parser::Value::String(s) => Value::String(s.as_str().to_string()),
                        crate::parser::Value::Date(d) => Value::Date(d.value),
                    };

                    out.values.entry(key.to_string()).or_default().push(value);
                }
                Item::Tag(project) => {
                    out.projects.push(project.to_string());
                }
            }
        }

        Ok(out)
    }
}

#[derive(Default)]
pub struct Collection {
    todos: Vec<Todo>,
}

impl Collection {
    #[cfg(feature = "std")]
    pub fn open_reader<T: std::io::Read>(
        read: T,
    ) -> Result<Collection, Box<dyn std::error::Error + Send + Sync>> {
        let buf_reader = std::io::BufReader::new(read);
        let lines = buf_reader.lines();

        let mut todos = Vec::default();

        for line in lines {
            let line = line?;
            let todo = parse(&line)?;
            todos.push(Todo::from(todo)?);
        }

        Ok(Collection { todos })
    }

    #[cfg(feature = "std")]
    pub fn write_writer<W: std::io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for todo in &self.todos {
            write!(writer, "{}\n", todo)?;
        }
        Ok(())
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Todo> {
        self.todos.get_mut(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Option<Todo> {
        if idx >= self.todos.len() {
            return None;
        }
        Some(self.todos.remove(idx))
    }

    pub fn projects(&self) -> BTreeSet<&str> {
        self.todos
            .iter()
            .map(|m| m.projects.iter())
            .flatten()
            .map(|m| m.as_str())
            .collect()
    }

    pub fn contexts(&self) -> BTreeSet<&str> {
        self.todos
            .iter()
            .map(|m| m.contexts.iter())
            .flatten()
            .map(|m| m.as_str())
            .collect()
    }

    pub fn create_todo(&mut self, todo: Todo) {
        self.todos.push(todo)
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Todo> {
        self.todos.iter()
    }

    pub fn len(&self) -> usize {
        self.todos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.todos.is_empty()
    }
}
