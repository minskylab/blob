use std::future;

use async_trait::async_trait;
use reqwest::Error;

#[async_trait]
pub trait LLMProcessor {
    async fn edit_call(
        self: Self,
        input: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Result<String, Error>;

    async fn completions_call(
        self: Self,
        prompt: impl Into<String>,
    ) -> future::Ready<Result<String, Error>>;
}
