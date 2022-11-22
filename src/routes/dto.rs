use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Email(String);

impl Into<String> for Email {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionFormData {
    pub name: String,
    pub email: String,
}
