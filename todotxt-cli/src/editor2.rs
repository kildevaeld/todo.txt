use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{Event, KeyCode, KeyModifiers, read},
    style::{self, Stylize},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Stdout, Write};

pub trait LimeItem {
    fn write(&self, output: &mut io::Stdout) -> io::Result<()>;
}

impl LimeItem for String {
    fn write(&self, output: &mut io::Stdout) -> io::Result<()> {
        output.queue(style::Print(self))?;
        Ok(())
    }
}

pub trait Collection {
    type Item: LimeItem;

    fn get(&self, idx: usize) -> Option<&Self::Item>;

    fn slice(&self, range: core::ops::Range<usize>) -> &[Self::Item];

    fn len(&self) -> usize;
}

impl<T: LimeItem> Collection for Vec<T> {
    type Item = T;
    fn get(&self, idx: usize) -> Option<&Self::Item> {
        (**self).get(idx)
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn slice(&self, range: core::ops::Range<usize>) -> &[Self::Item] {
        &self[range]
    }
}

pub trait Widget {
    fn update(&mut self, event: &Event) -> bool;
    fn render(&self, writer: &mut io::Stdout) -> color_eyre::Result<()>;
}

pub struct ListBox<T> {
    height: u16,
    top: u16,
    collection: T,
    current_index: u16,
    position: usize,
}

impl<T: Collection> ListBox<T> {
    pub fn new(collection: T, height: u16, top: u16) -> ListBox<T> {
        ListBox {
            height,
            top,
            collection,
            current_index: 0,
            position: 0,
        }
    }
}

impl<T: Collection> Widget for ListBox<T> {
    fn update(&mut self, event: &Event) -> bool {
        let render = match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if self.current_index == 0 {
                        if self.position > 0 {
                            self.position -= 1;
                            true
                        } else {
                            false
                        }
                    } else {
                        self.current_index -= 1;
                        true
                    }
                }
                KeyCode::Down => {
                    if self.current_index == self.height - 1 {
                        if (self.position as isize)
                            < (self.collection.len() as isize - self.height as isize)
                        {
                            self.position += 1;
                            true
                        } else {
                            false
                        }
                    } else {
                        self.current_index += 1;
                        true
                    }
                }
                _ => false,
            },
            _ => false,
        };

        render
    }

    fn render(&self, writer: &mut io::Stdout) -> color_eyre::Result<()> {
        let items = self
            .collection
            .slice(self.position..self.position + self.height as usize);

        writer.queue(cursor::MoveTo(0, self.top))?;
        let mut last = 0;
        for item in items {
            writer.queue(terminal::Clear(ClearType::CurrentLine))?;
            item.write(writer)?;
            writer.queue(cursor::MoveToNextLine(1))?;
            last += 1;
        }

        for _ in last..self.height {
            writer
                .queue(terminal::Clear(ClearType::CurrentLine))?
                .queue(cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }
}

pub fn run<W: Widget>(widget: &mut W) -> color_eyre::Result<()> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    let ret = run_inner(widget, &mut stdout);
    disable_raw_mode()?;

    ret
}

fn run_inner<W: Widget>(widget: &mut W, stdout: &mut Stdout) -> color_eyre::Result<()> {
    widget.render(stdout)?;

    loop {
        let event = read()?;

        match event {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            _ => {}
        }
    }

    Ok(())
}
