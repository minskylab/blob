use reqwest::{header::HeaderMap, Client, Error};
use serde_json::json;

use super::codex_responses::{CompletionResponse, EditResponse};

static CODEX_EDIT_API: &str = "https://api.openai.com/v1/engines/code-davinci-edit-001/edits";
static CODEX_COMPLETION_API: &str = "https://api.openai.com/v1/completions";

#[derive(Debug, Clone)]

pub struct CodexProcessor {
    http_client: Client,
    access_token: String,
}

impl CodexProcessor {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            http_client: Client::new(),
        }
    }

    pub async fn edit_call(
        self: Self,
        input: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Result<EditResponse, Error> {
        let endpoint = String::from(CODEX_EDIT_API);

        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&json! {
                {
                    "input": input.into(),
                    "instruction": instruction.into(),
                    "temperature": 0.2,
                    "top_p": 1,
                }
            })
            .send()
            .await?;

        let data = response.json::<EditResponse>().await?;

        // println!("{:?}", data);
        Ok(data)

        // Ok(data["choices"][0]["text"].as_str().unwrap().to_string())
    }

    pub async fn completions_call(
        self: Self,
        prompt: impl Into<String>,
        stop_words: Option<Vec<String>>,
    ) -> Result<CompletionResponse, Error> {
        let endpoint = String::from(CODEX_COMPLETION_API);

        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&json! {
                {
                    "model": "text-davinci-003",
                    "prompt": prompt.into(),
                    "max_tokens": 1000,
                    "temperature": 0.2,
                    "stop": stop_words,
                    // "top_p": 1,
                    // "n": 1,
                    // "stream": false,
                    // "logprobs": null,
                    // "stop": "\n"
                }
            })
            .send()
            .await?;

        let data = response.json::<CompletionResponse>().await?;
        Ok(data)
    }
}
