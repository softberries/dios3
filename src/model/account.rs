#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub access_key: String,
    pub secret_key: String,
    pub is_default: bool,
    pub default_region: String
}
impl Account {
    pub fn masked_secret_key(&self) -> String {
        let sk = &self.secret_key;
        if sk.len() >= 6 {
            let first = &sk[0..3];
            let last = &sk[sk.len() - 3..];
            format!("{first}*****{last}")
        } else {
            sk.clone()
        }
    }
}