use std::collections::VecDeque;
use std::io::ErrorKind;

pub(super) enum Enter {
    Before(ErrorKind),
    After(ErrorKind),
    Partial(bool),
    PanicAfterSubmit,
}

#[derive(Default)]
pub(super) struct Faults {
    pub enter: VecDeque<Enter>,
    pub hide_completions: bool,
    pub completion: Option<(Option<u64>, Option<i32>, Option<u32>)>,
}

impl Faults {
    pub fn transform(&mut self, (id, result, flags): (u64, i32, u32)) -> (u64, i32, u32) {
        match self.completion.take() {
            Some((new_id, new_result, new_flags)) => (
                new_id.unwrap_or(id),
                new_result.unwrap_or(result),
                new_flags.unwrap_or(flags),
            ),
            None => (id, result, flags),
        }
    }
}
