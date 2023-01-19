use super::Prompt;

use std::{
    fmt,
    io::{stdin, stdout, Write},
};

use termion::{
    color::{self, Black, Fg, LightBlack, LightGreen},
    cursor::Up,
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
    style::{self, Bold, Underline},
};

struct Option {
    text: String,
    checked: bool,
    selected: bool,
}

impl fmt::Display for Option {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: String;

        match self.checked {
            true => match self.selected {
                true => {
                    text = format!(
                        "  {}✓{} {Bold}{Underline}{}",
                        Fg(LightGreen),
                        Fg(color::Reset),
                        self.text
                    )
                }
                false => {
                    text = format!(
                        "  {}✓ {}{Bold}{}",
                        Fg(LightGreen),
                        Fg(color::Reset),
                        self.text
                    )
                }
            },
            false => match self.selected {
                true => text = format!("    {Bold}{Underline}{}", self.text),
                false => text = format!("    {}{Bold}{}", Fg(LightBlack), self.text),
            },
        }

        write!(f, "{}{}{}", text, style::Reset, Fg(color::Reset))
    }
}

pub struct MultipleChoiceList {
    options: Vec<Option>,
}

impl MultipleChoiceList {
    pub fn new(options: Vec<(&str, bool)>) -> Self {
        let mut object = Self { options: vec![] };

        options.into_iter().for_each(|(t, c)| object.add(t, c));

        object
    }

    pub fn add(&mut self, text: &str, checked: bool) {
        let option = Option {
            text: text.into(),
            checked,
            selected: if self.options.is_empty() { true } else { false },
        };

        self.options.push(option);
    }

    fn selected(&mut self) -> (usize, &mut Option) {
        self.options
            .iter_mut()
            .enumerate()
            .find(|(_, o)| o.selected)
            .unwrap()
    }
}

impl Prompt for MultipleChoiceList {
    type Output = Vec<String>;

    fn prompt(&mut self) -> Self::Output {
        if self.options.is_empty() {
            return vec![];
        }

        let big_list = self.options.len() >= 5;

        let mut stdout = stdout().into_raw_mode().unwrap();
        let mut stdin = stdin().keys();

        write!(
            stdout,
            "{Bold}Select what do you want to install{}{} - Space to select, Return to submit{}\r\n",
            style::Reset,
            Fg(Black),
            Fg(color::Reset)
        )
        .unwrap();

        loop {
            if big_list {
                let selected_index = self.selected().0;

                if selected_index < 2 {
                    for option in self.options.iter().take(5) {
                        write!(stdout, "{}\r\n", option).unwrap();
                    }
                } else if selected_index > self.options.len() - 3 {
                    for option in self.options.iter().rev().take(5).rev() {
                        write!(stdout, "{}\r\n", option).unwrap();
                    }
                } else {
                    let options = &self.options[selected_index - 2..=selected_index + 2];
                    for option in options {
                        write!(stdout, "{}\r\n", option).unwrap();
                    }
                }
            } else {
                for option in &self.options {
                    write!(stdout, "{}\r\n", option).unwrap();
                }
            }

            let input = stdin.next().unwrap().unwrap();

            match input {
                Key::Char('\n') => break,
                Key::Down | Key::Left => {
                    let (mut index, old) = self.selected();
                    old.selected = false;

                    if index == self.options.len() - 1 {
                        index = 0;
                    } else {
                        index += 1;
                    }

                    let new = self.options.get_mut(index).unwrap();
                    new.selected = true;
                }
                Key::Up | Key::Right => {
                    let (mut index, old) = self.selected();
                    old.selected = false;

                    if index == 0 {
                        index = self.options.len() - 1;
                    } else {
                        index -= 1;
                    }

                    let new = self.options.get_mut(index).unwrap();
                    new.selected = true;
                }
                Key::Char(' ') => {
                    let (_, option) = self.selected();
                    option.checked = !option.checked;
                }
                _ => {}
            }

            write!(
                stdout,
                "\r{}",
                Up(self.options.len().clamp(0, 5).try_into().unwrap())
            )
            .unwrap();
        }

        self.options
            .iter()
            .filter(|object| object.checked)
            .map(|object| object.text.clone())
            .collect()
    }
}
