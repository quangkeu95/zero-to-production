use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SubscriptionFormData {
    pub name: String,
    pub email: String,
}
