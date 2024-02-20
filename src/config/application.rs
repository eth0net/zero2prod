use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize)]
pub struct Config {
    pub host: String,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}
