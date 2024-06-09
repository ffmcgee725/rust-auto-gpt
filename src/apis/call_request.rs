use crate::models::general::llm::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::env;

// Call Large Language Model (i.e. GPT-4)
pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    let (api_key, api_org) = extract_keys();
    let url: &str = "https://api.openai.com/v1/chat/completions";
    let headers: HeaderMap = create_headers(&api_key, &api_org)?;

    let client: Client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    let chat_completion: ChatCompletion = ChatCompletion {
        model: "gpt-4".to_string(),
        messages,
        temperature: 0.1, // docs: https://platform.openai.com/docs/guides/text-generation/how-should-i-set-the-temperature-parameter
    };

    // Extract API Response
    let response: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    return Ok(response.choices[0].message.content.to_string());
}

fn extract_keys() -> (String, String) {
    let api_key: String =
        env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY missing in environment variables");
    let api_org: String =
        env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG missing in environment variables");
    return (api_key, api_org);
}

fn create_headers(
    api_key: &str,
    api_org: &str,
) -> Result<HeaderMap, Box<dyn std::error::Error + Send>> {
    let mut headers: HeaderMap = HeaderMap::new();

    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(api_org.to_string().as_str())
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    return Ok(headers);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_call_to_openai() {
        let message = Message {
            role: "user".to_string(),
            content: "Hi there, this is a test. Give me a short response.".to_string(),
        };

        call_gpt(vec![message]).await;
    }
}
