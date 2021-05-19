/// Detects whether the amount provided is singular or plural, and returns the correct form of the string
#[inline]
pub fn plural<'a>(amount: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if amount == 1 {
        singular
    } else {
        plural
    }
}
