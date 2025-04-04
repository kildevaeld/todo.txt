use inquire::Text;
use todotxt::{Collection, Todo, parser::parse};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .write(true)
        .read(true)
        .create(true)
        .open("todo.txt")?;

    let mut collection = Collection::open_reader(&mut file)?;

    let input = Text::new(">").prompt()?;

    let todo = Todo::from(parse(&input)?)?;

    collection.create_todo(todo);

    collection.write_writer(&mut file)?;

    Ok(())
}
