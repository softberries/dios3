#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub access_key: String,
    pub secret_key: String,
    pub is_default: String,
}
