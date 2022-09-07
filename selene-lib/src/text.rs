use std::{borrow::Cow, fmt::Write};

/// Detects whether the amount provided is singular or plural, and returns the correct form of the string
#[inline]
pub fn plural<'a>(amount: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if amount == 1 {
        singular
    } else {
        plural
    }
}

/// Produces a list that is comma separated, with the last item separated by "and"
pub fn english_list<'a>(list: &[&'a str]) -> Cow<'a, str> {
    match list.len() {
        0 => Cow::Owned(String::new()),
        1 => Cow::Borrowed(list[0]),
        2 => Cow::Owned(format!("{} and {}", list[0], list[1])),
        _ => {
            let mut string = String::new();

            for item in &list[..list.len() - 1] {
                write!(string, "{item}, ").ok();
            }

            write!(string, "and {}", list[list.len() - 1]).ok();

            Cow::Owned(string)
        }
    }
}
