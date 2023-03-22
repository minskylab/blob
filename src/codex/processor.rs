use anyhow::{anyhow, Result};
use reqwest::{header::HeaderMap, Client};
use serde_json::{from_str, json};

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
        self,
        input: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Result<EditResponse> {
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
        self,
        prompt: impl Into<String>,
        stop_words: Option<Vec<String>>,
    ) -> Result<CompletionResponse> {
        let endpoint = String::from(CODEX_COMPLETION_API);

        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", self.access_token).parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());

        let model_name = "text-davinci-003"; // "code-davinci-002"; // "text-davinci-003"

        let response = self
            .http_client
            .post(&endpoint)
            .headers(headers)
            .json(&json! {
                {
                    "model": model_name,
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

        let response_text = response.text().await.unwrap();

        let Ok(data) = from_str::<CompletionResponse>(&response_text) else {
            // let response_text = response_text;
            return Err(anyhow!(response_text));
        };

        Ok(data)
    }
}
