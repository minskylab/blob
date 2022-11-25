use reqwest::{header::HeaderMap, Client, Error};
use serde_json::{json, Value};

static CODEX_EDIT_API: &str = "https://api.openai.com/v1/engines/code-davinci-edit-001/edits";

#[derive(Debug, Clone)]

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

    pub async fn codex_call(
        self: Self,
        input: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Result<String, Error> {
        let endpoint = String::from(CODEX_EDIT_API);

        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());

        let body = json! {
           {
            "input": input.into(),
            "instruction": instruction.into(),
            "temperature": 0,
            "top_p": 1,
           }
        };

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let data = response.json::<Value>().await?;

        // println!("{:?}", data["choices"][0]["text"]);

        Ok(data["choices"][0]["text"].as_str().unwrap().to_string())
    }
}
