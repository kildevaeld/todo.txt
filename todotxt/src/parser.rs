use chrono::NaiveDate;
use udled::{
    Input, Lex, Tokenizer, WithSpan, any,
    token::{Char, Digit, EOF, Many, Opt, Spanned, Test},
};
use udled_tokenizers::{Bool, Float, Ident, Int, Str};

pub fn parse<'a>(input: &'a str) -> Result<Todo<'a>, udled::Error> {
    let mut input = Input::new(input);

    let todo = input.parse(TodoTokenizer)?;

    Ok(todo)
}

const SPACE: Many<char> = Many(' ');

struct TodoTokenizer;

impl Tokenizer for TodoTokenizer {
    type Token<'a> = Todo<'a>;
    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let done = reader.parse(Opt(CompletionParser))?;

        if done.is_some() {
            reader.eat(SPACE)?;
        }

        let priority = reader.parse(Opt(PriorityTokenizer))?;

        if priority.is_some() {
            reader.eat(SPACE)?;
        }

        let (created, completed) = if reader.peek(DateTokenizer)? {
            let mut created = reader.parse(DateTokenizer)?;
            reader.eat(SPACE)?;

            if reader.peek(DateTokenizer)? {
                let mut completed = reader.parse(DateTokenizer)?;
                core::mem::swap(&mut created, &mut completed);
                reader.eat(SPACE)?;
                (Some(created), Some(completed))
            } else {
                (Some(created), None)
            }
        } else {
            (None, None)
        };

        let description = reader.parse(DescriptionTokenizer)?;

        let mut items = Vec::default();

        loop {
            if reader.peek(any!('\n', EOF))? {
                break;
            }

            reader.eat(SPACE)?;

            let item = reader.parse(ItemTokenizer)?;

            items.push(item);
        }

        Ok(Todo {
            done: done.unwrap_or_default(),
            description,
            items,
            priority,
            completed,
            created,
        })
    }
}

struct DescriptionTokenizer;

impl Tokenizer for DescriptionTokenizer {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let start = reader.parse(Char)?.span();
        let mut end = start;

        loop {
            if reader.eof() {
                break;
            }

            if reader.peek(Test((
                Many(' '),
                any!(('@', Ident), ('+', Ident), (Ident, ':')),
            )))? {
                break;
            }

            end = reader.parse(Char)?.span();
        }

        if start == end {
            return Err(reader.error("Expected description"));
        }

        let span = start + end;

        Ok(Lex::new(span.slice(reader.source()).unwrap(), span))
    }
}

struct DateTokenizer;

impl Tokenizer for DateTokenizer {
    type Token<'a> = udled::Item<NaiveDate>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let span = reader.parse(Spanned((
            (Digit(10), Digit(10), Digit(10), Digit(10)),
            '-',
            (Digit(10), Digit(10)),
            '-',
            (Digit(10), Digit(10)),
        )))?;

        let str = span.slice(reader.source()).unwrap();

        Ok(udled::Item::new(
            NaiveDate::parse_from_str(str, "%Y-%m-%d")
                .map_err(|err| reader.error(err.to_string()))?,
            span,
        ))
    }
}

struct PriorityTokenizer;

impl Tokenizer for PriorityTokenizer {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let (_, lex, _) = reader.parse(('(', Char, ')'))?;

        Ok(lex)
    }
}

struct CompletionParser;

impl Tokenizer for CompletionParser {
    type Token<'a> = bool;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        reader.parse(any!("x", "X")).map(|_| true)
    }
}

struct ItemTokenizer;

impl Tokenizer for ItemTokenizer {
    type Token<'a> = Item<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        if reader.peek('+')? {
            let (_, ident) = reader.parse(('+', Ident))?;
            Ok(Item::Tag(ident))
        } else if reader.peek('@')? {
            let (_, ident) = reader.parse(('@', Ident))?;
            Ok(Item::Context(ident))
        } else {
            let (key, _, value) = reader.parse((Ident, (SPACE, ':', SPACE), ValueParser))?;
            Ok(Item::KeyVal { key, value })
        }
    }
}

struct ValueParser;

impl Tokenizer for ValueParser {
    type Token<'a> = Value<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        if reader.peek(DateTokenizer)? {
            Ok(Value::Date(reader.parse(DateTokenizer)?))
        } else if reader.peek(Float)? {
            Ok(Value::Float(reader.parse(Float)?))
        } else if reader.peek(Int)? {
            Ok(Value::Int(reader.parse(Int)?))
        } else if reader.peek(Bool)? {
            Ok(Value::Bool(reader.parse(Bool)?))
        } else if reader.peek(Ident)? {
            Ok(Value::String(reader.parse(Ident)?))
        } else {
            Ok(Value::String(reader.parse(Str)?))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Item<'a> {
    Tag(Lex<'a>),
    Context(Lex<'a>),
    KeyVal { key: Lex<'a>, value: Value<'a> },
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    Date(udled::Item<NaiveDate>),
    String(Lex<'a>),
    Int(udled::Item<i128>),
    Float(udled::Item<f64>),
    Bool(udled::Item<bool>),
}

#[derive(Debug, Clone)]
pub struct Todo<'a> {
    pub done: bool,
    pub priority: Option<Lex<'a>>,
    pub description: Lex<'a>,
    pub items: Vec<Item<'a>>,
    pub created: Option<udled::Item<NaiveDate>>,
    pub completed: Option<udled::Item<NaiveDate>>,
}
