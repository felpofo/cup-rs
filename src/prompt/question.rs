use super::Prompt;
use std::io::{stdin, stdout, BufRead, Write};
use termion::{
    color::{self, Black, Fg},
    style::{self, Bold},
};

pub struct Question {
    question: String,
    default: String,
}

impl Question {
    pub fn new(question: &str) -> Self {
        let question = question.into();

        Self {
            question,
            default: "".into(),
        }
    }

    pub fn default(self, default: &str) -> Self {
        let default = default.into();

        Self { default, ..self }
    }
}

impl Prompt for Question {
    type Output = String;

    fn prompt(&mut self) -> Self::Output {
        println!(
            "{Bold}{}{}{} - Return to submit{}",
            self.question,
            style::Reset,
            Fg(Black),
            Fg(color::Reset)
        );

        if !self.default.is_empty() {
            print!(
                "{}(Default: {}):{} ",
                Fg(Black),
                &self.default,
                Fg(color::Reset)
            );
        } else {
            print!("Enter value: ");
        }

        stdout().flush().unwrap();

        let answer = stdin()
            .lock()
            .lines()
            .next()
            .unwrap()
            .map(|line| line.trim_end().to_owned())
            .unwrap();

        if answer.is_empty() {
            return self.default.clone();
        }

        answer
    }
}
