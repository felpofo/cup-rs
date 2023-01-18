mod multiple_choice_list;
mod question;

pub use multiple_choice_list::MultipleChoiceList;
pub use question::Question;

pub trait Prompt {
    type Output;

    fn prompt(&mut self) -> Self::Output;
}
