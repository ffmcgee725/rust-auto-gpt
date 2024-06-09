use async_trait::async_trait;
use reqwest::Client;
use std::{
    process::{Command, Stdio},
    time::Duration,
};
use tokio::time;

use crate::{
    ai_functions::backend::{
        print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
        print_rest_api_endpoints,
    },
    helpers::{
        command_line::{confirm_safe_code, PrintCommand},
        utils::{
            ai_task_request, check_status_code, read_code_template_content,
            read_executable_main_content, save_api_endpoints, save_backend_code,
            WEB_SERVER_PROJECT_PATH,
        },
    },
    models::agent_basic::{
        basic_agent::{AgentState, BasicAgent},
        basic_traits::BasicTraits,
    },
};

use super::agent_trait::{FactSheet, RouteObject, SpecialFunctions};

#[derive(Debug)]
pub struct AgentBackendDeveloper {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDeveloper {
    pub fn new() -> Self {
        return Self {
            attributes: BasicAgent {
                objective: "Develops the backend code for web server and json database."
                    .to_string(),
                position: "Backend Developer".to_string(),
                state: AgentState::Discovery,
                memory: Vec::new(),
            },
            bug_errors: None,
            bug_count: 0,
        };
    }

    pub async fn call_initial_backend_code(&mut self, fact_sheet: &mut FactSheet) {
        let code_template_str: String = read_code_template_content();

        let msg_context: String = format!(
            "CODE TEMPLATE: {} \n PROJECT DESCRIPTION: {}",
            code_template_str, fact_sheet.project_description
        );

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        self.git_commit_code(ai_response, fact_sheet);
    }

    pub async fn call_improved_backend_code(&mut self, fact_sheet: &mut FactSheet) {
        let msg_context: String = format!(
            "CODE TEMPLATE: {:?} \n PROJECT DESCRIPTION: {:?}",
            fact_sheet.backend_code, fact_sheet.project_description
        );
        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        self.git_commit_code(ai_response, fact_sheet);
    }

    pub async fn call_fix_code_bugs(&mut self, fact_sheet: &mut FactSheet) {
        let msg_context: String = format!(
            "BROKEN CODE: {:?} \n ERROR BUGS: {:?} \n 
            THIS FUNCTION JUST OUTPUTS THE CODE. JUST OUTPUT THE CODE.",
            fact_sheet.backend_code, self.bug_errors
        );

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        self.git_commit_code(ai_response, fact_sheet);
    }

    pub async fn call_extract_rest_api_endpoints(&self) -> String {
        let backend_code: String = read_executable_main_content();

        let msg_context: String = format!("CODE INPUT: {:?} \n", backend_code);

        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;

        return ai_response;
    }

    pub fn ensure_ai_safety(&mut self) {
        PrintCommand::UnitTest.print_agent_message(
            &self.attributes.get_position().as_str(),
            "Backend Code Unit Testing: Ensuring code safety",
        );

        let is_safe_code: bool = confirm_safe_code();
        if !is_safe_code {
            panic!("Aborted flow!")
        }
    }

    pub fn build_code(&mut self) -> std::process::Output {
        PrintCommand::UnitTest.print_agent_message(
            &self.attributes.get_position().as_str(),
            "Backend Code Unit Testing: Building project",
        );

        let build_backend_server: std::process::Output = Command::new("cargo")
            .arg("build")
            .current_dir(WEB_SERVER_PROJECT_PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to build backend application");

        return build_backend_server;
    }

    pub async fn run_code(&mut self, check_endpoints: &Vec<RouteObject>) {
        PrintCommand::UnitTest.print_agent_message(
            &self.attributes.get_position().as_str(),
            "Backend Code Unit Testing: Starting web server...",
        );

        let mut run_backend_server: std::process::Child = Command::new("cargo")
            .arg("run")
            .current_dir(WEB_SERVER_PROJECT_PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run backend application");

        PrintCommand::UnitTest.print_agent_message(
            &self.attributes.get_position().as_str(),
            "Backend Code Unit Testing: Launching tests on server in 5 seconds...",
        );

        time::sleep(Duration::from_secs(5)).await;

        let client: Client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        for endpoint in check_endpoints {
            PrintCommand::UnitTest.print_agent_message(
                &self.attributes.get_position().as_str(),
                format!("Testing endpoint {}", endpoint.route).as_str(),
            );

            let url: String = format!("http://127.0.0.1:8080{}", endpoint.route);

            match check_status_code(&client, &url).await {
                Ok(status_code) => {
                    if status_code != 200 {
                        PrintCommand::Issue.print_agent_message(
                            &self.attributes.get_position().as_str(),
                            format!(
                                "WARNING: Failed to call backend url endpoint {}",
                                endpoint.route
                            )
                            .as_str(),
                        );
                    }
                }
                Err(e) => {
                    // kill process in port $(lsof -t -i:8080)
                    run_backend_server
                        .kill()
                        .expect("Failed to kill backend web server");
                    PrintCommand::Issue.print_agent_message(
                        &self.attributes.get_position().as_str(),
                        format!(
                            "WARNING: Failed to call backend url endpoint {}",
                            format!("Error checking backend {}", e)
                        )
                        .as_str(),
                    );
                }
            }

            run_backend_server
                .kill()
                .expect("Failed to kill backend web server on completion");
        }
    }

    pub fn handle_errs(&mut self, built_backend_server: std::process::Output) {
        let error_array: Vec<u8> = built_backend_server.stderr;
        let error_string: String = String::from_utf8(error_array).unwrap();

        self.bug_count += 1;
        self.bug_errors = Some(error_string);

        if self.bug_count > 2 {
            PrintCommand::Issue.print_agent_message(
                &self.attributes.get_position().as_str(),
                "Backend Code Unit Testing: Too many bugs found in code, shutting down..",
            );
            panic!("Error: Too many bugs to successfully build and run code");
        }
    }

    pub async fn extract_and_test_rest_api_endpoints(
        &mut self,
        api_endpoints_str: &str,
    ) -> Vec<RouteObject> {
        let api_endpoints: Vec<RouteObject> = serde_json::from_str(&api_endpoints_str)
            .expect(format!("Failed to decode API Endpoints: {}", api_endpoints_str).as_str());

        // We only check non complex routes (i.e. no dynamic params)
        let endpoints_to_check: Vec<RouteObject> = api_endpoints
            .iter()
            .filter(|&route_object| {
                route_object.method == "get" && route_object.is_route_dynamic == false
            })
            .cloned()
            .collect();

        return endpoints_to_check;
    }

    fn git_commit_code(&mut self, code: String, fact_sheet: &mut FactSheet) {
        save_backend_code(&code);
        fact_sheet.backend_code = Some(code);
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDeveloper {
    fn get_agent_position(&mut self) -> String {
        return self.attributes.get_position().clone();
    }

    async fn execute(
        &mut self,
        fact_sheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // /!\ WARNING BE CAREFUL OF INFINITE LOOPS => INFINITE GPT API COSTS /!\
        while self.attributes.get_state() != &AgentState::Finished {
            match self.attributes.get_state() {
                &AgentState::Discovery => {
                    self.call_initial_backend_code(fact_sheet).await;
                    self.attributes.update_state(AgentState::Working);
                    continue;
                }

                &AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improved_backend_code(fact_sheet).await;
                    } else {
                        self.call_fix_code_bugs(fact_sheet).await;
                    }
                    self.attributes.update_state(AgentState::UnitTesting);
                    continue;
                }

                &AgentState::UnitTesting => {
                    self.ensure_ai_safety();
                    let built_backend_server: std::process::Output = self.build_code();

                    if built_backend_server.status.success() {
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_message(
                            &self.attributes.get_position().as_str(),
                            "Backend Code Unit Testing: Test server build successful!!",
                        );
                    } else {
                        self.handle_errs(built_backend_server);
                        self.attributes.update_state(AgentState::Working);
                        continue;
                    }

                    let api_endpoints_str: String = self.call_extract_rest_api_endpoints().await;

                    let endpoints_to_check: Vec<RouteObject> = self
                        .extract_and_test_rest_api_endpoints(&api_endpoints_str)
                        .await;

                    let api_endpoint_full_schema: Vec<RouteObject> =
                        serde_json::from_str(&api_endpoints_str).expect(
                            format!("Failed to decode API endpoints: {}", &api_endpoints_str)
                                .as_str(),
                        );

                    fact_sheet.api_endpoint_schema = Some(api_endpoint_full_schema);

                    self.run_code(&endpoints_to_check).await;
                    save_api_endpoints(&api_endpoints_str);

                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.get_position().as_str(),
                        "Backend Code Unit Testing: Complete!!!",
                    );

                    // /!\ WARNING: SETTING STATE TO FINISHED IS ESSENTIAL => BE CAREFUL OF INFINITE LOOPS /!\
                    self.attributes.update_state(AgentState::Finished);
                }
                _ => {}
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::models::agents::agent_trait::ProjectScope;

    use super::*;

    #[tokio::test]
    async fn tests_writing_backend_code() {
        let mut agent = AgentBackendDeveloper::new();

        // Basic Fact Sheet
        let mut fact_sheet: FactSheet = FactSheet {
            project_description:
                "build a website that fetches and displays random images of cats from an external API".to_string(),
            project_scope: Some(ProjectScope {
                is_crud_required: false,
                is_user_login_and_logout: false,
                is_external_urls_required: true,
            }),
            external_urls: Some(vec![
                "https://api.thecatapi.com/v1/images/search".to_string(),
            ]),
            backend_code: None,
            api_endpoint_schema: None,
        };

        agent
            .execute(&mut fact_sheet)
            .await
            .expect("Unable to execute Backend Developer Agent");

        assert!(fact_sheet.backend_code.is_some());
        assert!(fact_sheet.api_endpoint_schema.is_some());
    }
}
