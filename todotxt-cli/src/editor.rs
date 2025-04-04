use crossterm::{
    QueueableCommand, cursor,
    event::{Event, KeyCode, read},
    style::{self, Stylize},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use todotxt::Collection;

pub fn run(collection: &mut Collection) -> color_eyre::Result<()> {
    let mut editor = Editor::new(5, collection)?;

    editor.run()?;

    Ok(())
}

struct Editor<'a> {
    window_height: u16,
    buffer: String,
    w: io::Stdout,
    top: u16,
    collection: &'a mut Collection,
}

impl<'a> Editor<'a> {
    fn new(window_height: u16, collection: &'a mut Collection) -> color_eyre::Result<Editor<'a>> {
        for x in 0..(window_height + 2) {
            println!();
        }

        let (_, current_row) = cursor::position()?;
        let start = current_row - (window_height + 2);

        Ok(Editor {
            window_height,
            buffer: String::new(),
            w: io::stdout(),
            top: start,
            collection,
        })
    }

    fn run(&mut self) -> color_eyre::Result<()> {
        enable_raw_mode()?;
        let ret = self.run_inner();
        disable_raw_mode()?;
        ret
    }

    fn run_inner(&mut self) -> color_eyre::Result<()> {
        self.set_input("")?;
        self.render_help()?;
        self.render_list(0, 0)?;

        self.w.queue(cursor::MoveTo(1, self.top))?;

        self.w.flush()?;

        let mut current_row = 0u16;
        let mut pointer_idx = 0usize;

        loop {
            let event = read()?;

            match event {
                Event::Key(key) => {
                    if key.code == KeyCode::Esc {
                        break;
                    }
                    match key.code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Up => {
                            if current_row == 0 {
                                if pointer_idx > 0 {
                                    pointer_idx -= 1;
                                }
                            } else {
                                current_row -= 1;
                            }

                            self.render_list(pointer_idx, current_row)?;
                        }
                        KeyCode::Down => {
                            if current_row == self.window_height - 1 {
                                if (pointer_idx as isize)
                                    < (self.collection.len() as isize - self.window_height as isize)
                                {
                                    pointer_idx += 1;
                                }
                            } else {
                                current_row += 1;
                            }

                            self.render_list(pointer_idx, current_row)?;
                        }
                        KeyCode::Char('d') => {
                            self.collection.remove(pointer_idx + current_row as usize);
                            self.render_list(pointer_idx, current_row)?;
                        }
                        KeyCode::Char('c') => {
                            let todo = self
                                .collection
                                .get_mut(pointer_idx + current_row as usize)
                                .unwrap();
                            if todo.done {
                                todo.completed = None;
                                todo.done = false;
                            } else {
                                todo.completed = Some(chrono::Local::now().date_naive());
                                todo.done = true;
                            }
                            self.render_list(pointer_idx, current_row)?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            self.w.queue(cursor::MoveTo(1, self.top))?.flush()?;
        }

        Ok(())
    }

    fn clear(&mut self) -> color_eyre::Result<()> {
        Ok(())
    }

    fn render_help(&mut self) -> color_eyre::Result<()> {
        self.w
            .queue(cursor::MoveTo(0, self.top + self.window_height + 1))?
            .queue(terminal::Clear(ClearType::CurrentLine))?
            .queue(style::Print("Enter (d)elete, (c)omplete, (e)scape."))?;
        Ok(())
    }

    fn render_list(&mut self, position: usize, current: u16) -> color_eyre::Result<()> {
        let items = self
            .collection
            .iter()
            .skip(position)
            .take(self.window_height as usize);

        self.w.queue(cursor::MoveTo(0, self.top))?;
        let mut last = 0;
        for (idx, item) in items.enumerate() {
            self.w
                .queue(cursor::MoveToNextLine(1))?
                .queue(terminal::Clear(ClearType::CurrentLine))?;

            if idx == current as usize {
                self.w.queue(style::PrintStyledContent(
                    ">".attribute(style::Attribute::Bold),
                ))?;
            } else {
                self.w.queue(cursor::MoveRight(1))?;
            }

            self.w
                .queue(cursor::MoveRight(1))?
                .queue(if item.done {
                    style::Print("[x]")
                } else {
                    style::Print("[ ]")
                })?
                .queue(style::Print(&item.description))?;

            for project in &item.projects {
                self.w.queue(style::PrintStyledContent(
                    format!(" +{project}")
                        .with(style::Color::Magenta)
                        .attribute(style::Attribute::Bold),
                ))?;
            }

            last += 1;
        }

        for _ in last..self.window_height {
            self.w
                .queue(cursor::MoveToNextLine(1))?
                .queue(terminal::Clear(ClearType::CurrentLine))?;
        }

        Ok(())
    }

    fn set_input(&mut self, input: &str) -> color_eyre::Result<()> {
        self.buffer = input.to_string();
        self.w
            .queue(cursor::MoveTo(0, self.top))?
            .queue(style::Print("> "))?
            .queue(style::Print(input))?;
        Ok(())
    }
}
