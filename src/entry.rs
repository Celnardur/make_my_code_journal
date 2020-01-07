use chrono::NaiveDateTime;

// journal entry that you wrote yourself
#[allow(dead_code)]
pub struct Entry {
    time: NaiveDateTime,
    title: Option<String>,
    text: String,
}
