use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Artwork {
    pub name: String,
    #[serde(default)]
    pub info: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ArtworksResponse {
    #[serde(rename = "type")]
    response_type: String,
    name: String,
    pub contents: Vec<Artwork>,
}
