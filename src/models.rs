use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CrawlUrlRequest {
    pub url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchAndroidRequest {
    pub query: String,
    pub max_page: Option<u32>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
pub struct Link {
    pub href: String,
    pub text: String,
}