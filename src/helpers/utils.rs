use std::fs;

use super::command_line::PrintCommand;
use crate::{apis::call_request::call_gpt, models::general::llm::Message};
use reqwest::Client;
use serde::de::DeserializeOwned;

pub const WEB_SERVER_PROJECT_PATH: &str = "src/templates/web_server";
const CODE_TEMPLATE_PATH: &str = "src/templates/web_server/src/code_template.rs";
const EXEC_MAIN_PATH: &str = "src/templates/web_server/src/main.rs";
const API_SCHEMA_JSON: &str = "schemas/api_schema.json";

// Extend AI function to encourage certain specific output
pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, arguments: &str) -> Message {
    let ai_func_string = ai_func(arguments);

    // Extend the string to encourage only printing the output
    let content = format!(
        "FUNCTION {}
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentary. Here is the input for the function: {}.
    Print out what the function will return.",
        ai_func_string, arguments
    );

    return Message {
        role: "system".to_string(),
        content,
    };
}

// Performs call to LLM GPT
pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let extended_message = extend_ai_function(function_pass, &msg_context);

    PrintCommand::AICall.print_agent_message(agent_position, agent_operation);

    let llm_response: Result<String, Box<dyn std::error::Error + Send>> =
        call_gpt(vec![extended_message.clone()]).await;

    return match llm_response {
        Ok(result) => result,
        Err(_) => call_gpt(vec![extended_message.clone()])
            .await
            .expect("Failed twice to call Open AI"),
    };
}

// Performs call to LLM GPT -- Decoded -> Parses JSON string to parametrized structure
pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response: String =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;

    let decoded_response: T = serde_json::from_str(llm_response.as_str())
        .expect("Failed to decode AI response from serde_json");

    return decoded_response;
}

pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response: reqwest::Response = client.get(url).send().await?;
    return Ok(response.status().as_u16());
}

pub fn read_code_template_content() -> String {
    let path: String = String::from(CODE_TEMPLATE_PATH);
    return fs::read_to_string(path).expect("Failed to read code template");
}

pub fn read_executable_main_content() -> String {
    let path: String = String::from(EXEC_MAIN_PATH);
    return fs::read_to_string(path).expect("Failed to read code template");
}

pub fn save_backend_code(content: &str) -> () {
    let path: String = String::from(EXEC_MAIN_PATH);
    fs::write(path, content).expect("Failed to write main.rs file");
}

pub fn save_api_endpoints(api_endpoints: &String) {
    let path: String = String::from(API_SCHEMA_JSON);
    fs::write(path, api_endpoints).expect("Failed to write API endpoints to file");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::managing::convert_user_input_to_goal;

    #[test]
    fn tests_extending_ai_function() {
        let extended_message: Message =
            extend_ai_function(convert_user_input_to_goal, "dummy variable");
        dbg!(&extended_message);
        assert_eq!(extended_message.role, "system".to_string());
    }

    #[tokio::test]
    async fn test_ai_request_task() {
        let ai_func_param: String =
            "Build me a webserver for making stock price API requests.".to_string();
        let res: String = ai_task_request(
            ai_func_param,
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        )
        .await;

        dbg!(&res);
        assert!(res.len() > 20);
    }
}
