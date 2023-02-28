use super::Prompt;
use std::io::{stdin, stdout, BufRead, Write};
use termion::style::{self, Bold};

pub struct Question {
    question: String,
    default: String,
}

impl Question {
    pub fn new(question: &str, default: &str) -> Self {
        let question = question.into();
        let default = default.into();

        Self { question, default }
    }
}

impl Prompt for Question {
    type Output = String;

    fn prompt(&mut self) -> Self::Output {
        print!("{Bold}{}{}", self.question, style::Reset);

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
