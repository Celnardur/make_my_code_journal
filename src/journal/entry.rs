use chrono::NaiveDateTime;

// journal entry that you wrote yourself
pub struct Entry {
    time: NaiveDateTime,
    title: Option<String>,
    text: String,
}

