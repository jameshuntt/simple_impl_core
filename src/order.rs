// --------------------
// order helpers
// --------------------

#[derive(Debug, Clone)]
pub enum Order {
    Key(i32),
    First,
    Last,
    Before(String),
    After(String),
}