use std::{collections::HashMap, io::BufRead};

use chrono::NaiveDate;

use crate::parser;

pub struct Todo {
    description: String,
    created: Option<NaiveDate>,
    updated: Option<NaiveDate>,
    contexts: Vec<String>,
    projects: Vec<String>,
    completed: bool,
    values: HashMap<String, Vec<String>>,
}

pub struct Collection {
    todos: Vec<Todo>,
}

impl Collection {
    pub fn open_writer<T: std::io::Read>(read: T) {
        let buf_reader = std::io::BufReader::new(read);
        let mut lines = buf_reader.lines();

        for line in lines {}
    }
}
