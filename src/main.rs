use std::io::stdin;

use chrono::Local;
use regex::{Regex, RegexBuilder};

fn main() {
    let console_input = stdin();

    println!("Hello, world! I'm a cool chatbot! What is your name?");
    let mut name = String::new();
    console_input.read_line(&mut name).unwrap();

    println!("Hello, {}, ask me something!", name.trim());

    let time_matcher = Matcher::word("time")
        .or("date")
        .or_m(Matcher::word("date").and("what"));
    let time_responder = move |input: String| {
        if time_matcher.matches(&input) {
            Some(format!("It's currently {}", Local::now().to_rfc2822()))
        } else {
            None
        }
    };

    let bot = Bot::new(name)
        .add_responder(
            Matcher::word("who").or("what").and("you"),
            "I'm a cool chatbot.",
        )
        .add_responder(Matcher::word("write").and("essay"), "I don't like essays.")
        .add_responder(Matcher::word("write"), "I'm illiterate.")
        .add_responder(
            Matcher::word("class"),
            "This bot was made in Computer Science!",
        )
        .add_responder(Matcher::word("is").and("?"), "No it is not.")
        .add_responder(
            Matcher::word("number")
                .or("how many")
                .and_m(Matcher::text("computer")),
            "28",
        )
        .add_responder(
            Matcher::word("teacher").and("computer"),
            "Ms. Lau teaches computer science.",
        )
        .add_responder(Matcher::word("extra credit").and("not"), "You're crazy")
        .add_responder(
            Matcher::word("extra credit"),
            "This group should get extra credit",
        )
        .add_responder(
            Matcher::word("group").and("chatbot"),
            "My creators are Dash, Patrick, and Josh",
        )
        .add_responder(
            Matcher::word("made")
                .or("created")
                .and("why")
                .and("?")
                .and("you"),
            "I was created for an APCSA assignment in Java.",
        )
        .add_responder(Matcher::word("are you sure"), "Yes, I'm sure.")
        .add_dyn_responder(Box::new(time_responder) as _)
        .add_responder(Matcher::word("huh").or("what"), "Didn't you hear?");

    let mut input = String::new();

    loop {
        console_input.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        match bot.respond(&input) {
            Response::Exit => break,
            Response::Failed => println!("Sorry, I don't understand."),
            Response::Text(text) => println!("{text}"),
        }
        input.clear();
    }
}

trait DynResponder {
    fn respond(&self, to: &str) -> Option<String>;
}
impl<T> DynResponder for T
where
    T: Fn(String) -> Option<String>,
{
    fn respond(&self, to: &str) -> Option<String> {
        self(to.to_string())
    }
}

struct Bot {
    name: String,
    responders: Vec<Responder>,
}
impl Bot {
    fn new(name: String) -> Self {
        Self {
            name,
            responders: Vec::new(),
        }
    }
    fn respond(&self, query: &str) -> Response {
        if query.to_lowercase().contains("exit") {
            return Response::Exit;
        }
        self.responders
            .iter()
            .filter_map(|r| r.respond(query))
            // .filter(|r| r.matcher.matches(query))
            // .map(|r| Response::Text(r.response.clone()))
            .map(Response::Text)
            .next()
            .unwrap_or(Response::Failed)
    }
    fn add_responder(mut self, matcher: Matcher, response: impl ToString) -> Self {
        self.responders.push(Responder::Static {
            matcher,
            response: response.to_string(),
        });
        self
    }
    fn add_dyn_responder(mut self, responder: Box<dyn DynResponder>) -> Self {
        self.responders.push(Responder::Dynamic(responder));
        self
    }
}

enum Response {
    Exit,
    Text(String),
    Failed,
}

enum Responder {
    Static { matcher: Matcher, response: String },
    Dynamic(Box<dyn DynResponder>),
}
impl Responder {
    fn respond(&self, to: &str) -> Option<String> {
        match self {
            Self::Static { matcher, response } => {
                if matcher.matches(to) {
                    Some(response.clone())
                } else {
                    None
                }
            }
            Self::Dynamic(dyn_responder) => dyn_responder.respond(to),
        }
    }
}

#[allow(dead_code)]
impl Matcher {
    fn word(text: &str) -> Self {
        let regex = RegexBuilder::new(&format!(
            "(?i)\\b{}\\b",
            regex::escape(&text.to_lowercase())
        ))
        .case_insensitive(true)
        .build()
        .unwrap();
        Self::Pattern(regex)
    }
    fn text(text: &str) -> Self {
        Self::Pattern(Regex::new(&regex::escape(text)).unwrap())
    }
    fn regex(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self::Pattern(Regex::new(pattern)?))
    }
    fn and_m(self, other: Matcher) -> Self {
        Matcher::And(Box::new(self), Box::new(other))
    }
    fn and(self, other: &str) -> Self {
        Matcher::And(Box::new(self), Box::new(Matcher::word(other)))
    }
    fn or_m(self, other: Matcher) -> Self {
        Matcher::Or(Box::new(self), Box::new(other))
    }
    fn or(self, other: &str) -> Self {
        Matcher::Or(Box::new(self), Box::new(Matcher::word(other)))
    }
    fn not(self) -> Self {
        Matcher::Not(Box::new(self))
    }
    fn matches(&self, against: &str) -> bool {
        match self {
            Matcher::Pattern(regex) => regex.is_match(against),
            Matcher::Or(a, b) => a.matches(against) || b.matches(against),
            Matcher::And(a, b) => a.matches(against) && b.matches(against),
            Matcher::Not(not) => !not.matches(against),
        }
    }
}

enum Matcher {
    Pattern(Regex),
    And(Box<Matcher>, Box<Matcher>),
    Or(Box<Matcher>, Box<Matcher>),
    Not(Box<Matcher>),
}
