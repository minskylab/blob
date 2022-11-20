use reqwest::{
    get,
    header::{HeaderMap, HeaderValue},
    Client, Error,
};
use std::{any::Any, collections::HashMap};
// use serde_json::json;

static CODEX_EDIT_API: &str = "https://api.openai.com/v1/edits";

pub struct Processor {
    http_client: Client,
    access_token: String,
}

impl Processor {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            http_client: Client::new(),
        }
    }

    pub async fn codex_call(self: Self, prompt: String) -> Result<(), Error> {
        let endpoint = String::from(CODEX_EDIT_API);

        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());

        let mut body = HashMap::new();
        body.insert("model", "text-davinci-edit-001");
        // body.insert("max_tokens", 50);
        // body.insert("temperature", 0.7);
        // body.insert("top_p", 0.9);
        // body.insert("n", 3);
        // body.insert("stream", false);
        // body.insert("logprobs", 0);
        // body.insert("stop", "");

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        println!("{:?}", response);

        return Ok(());
    }
}
