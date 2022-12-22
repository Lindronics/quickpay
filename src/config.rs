use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub client_id: String,
    pub client_secret: String,
    pub client_kid: String,
    pub client_private_key: String,
    pub redirect_uri: String,
}
