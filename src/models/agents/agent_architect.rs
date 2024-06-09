use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

use crate::{
    ai_functions::architect::{print_project_scope, print_site_urls},
    helpers::{
        command_line::PrintCommand,
        utils::{ai_task_request_decoded, check_status_code},
    },
    models::agent_basic::{
        basic_agent::{AgentState, BasicAgent},
        basic_traits::BasicTraits,
    },
};

use super::agent_trait::{FactSheet, ProjectScope, SpecialFunctions};

#[derive(Debug)]
pub struct AgentSolutionArchitect {
    attributes: BasicAgent,
}

impl AgentSolutionArchitect {
    pub fn new() -> Self {
        return Self {
            attributes: BasicAgent {
                objective: "Gathers information and design solution for website development."
                    .to_string(),
                position: "Solutions Architect".to_string(),
                state: AgentState::Discovery,
                memory: Vec::new(),
            },
        };
    }

    pub async fn call_project_scope(&mut self, fact_sheet: &mut FactSheet) -> ProjectScope {
        let msg_context: String = format!("{:?}", fact_sheet.project_description);
        let ai_response: ProjectScope = ai_task_request_decoded::<ProjectScope>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_project_scope),
            print_project_scope,
        )
        .await;
        fact_sheet.project_scope = Some(ai_response.clone());

        // /!\ WE MAKE SURE TO CHANGE STATE TO FINISHED, OTHERWISE WE RISK INFINITE LOOPS => INFINITE GPT API COSTS /!\
        self.attributes.update_state(AgentState::Finished);
        return ai_response;
    }

    pub async fn call_determine_external_urls(
        &mut self,
        fact_sheet: &mut FactSheet,
        msg_context: String,
    ) -> () {
        let ai_response: Vec<String> = ai_task_request_decoded::<Vec<String>>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_site_urls),
            print_site_urls,
        )
        .await;
        fact_sheet.external_urls = Some(ai_response);
        self.attributes.update_state(AgentState::UnitTesting);
    }

    pub async fn handle_discovery_state(&mut self, fact_sheet: &mut FactSheet) -> () {
        let project_scope: ProjectScope = self.call_project_scope(fact_sheet).await;
        if project_scope.is_external_urls_required {
            self.call_determine_external_urls(fact_sheet, fact_sheet.project_description.clone())
                .await;
            self.attributes.update_state(AgentState::UnitTesting);
        }
    }

    pub async fn handle_unit_testing_state(&mut self, fact_sheet: &mut FactSheet) -> () {
        let mut exclude_urls: Vec<String> = Vec::new();
        let client: Client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let urls: &Vec<String> = fact_sheet
            .external_urls
            .as_ref()
            .expect("No URL object on fact sheet.");

        for url in urls {
            let endpoint_string: String = format!("Testing URL Endpoint: {}", url);
            PrintCommand::UnitTest.print_agent_message(&self.attributes.position, &endpoint_string);

            match check_status_code(&client, url).await {
                Ok(status_code) => {
                    if status_code != 200 {
                        exclude_urls.push(url.clone());
                    }
                }
                Err(e) => println!("Error checking {}:, {}", url, e),
            }
        }

        if exclude_urls.len() > 0 {
            let new_urls: Vec<String> = fact_sheet
                .external_urls
                .as_ref()
                .unwrap()
                .iter()
                .filter(|url| !exclude_urls.contains(&url))
                .cloned()
                .collect();
            fact_sheet.external_urls = Some(new_urls);
        }

        // /!\ WE MAKE SURE TO CHANGE STATE TO FINISHED, OTHERWISE WE RISK INFINITE LOOPS => INFINITE GPT API COSTS /!\
        self.attributes.update_state(AgentState::Finished);
    }
}

#[async_trait]
impl SpecialFunctions for AgentSolutionArchitect {
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
                &AgentState::Discovery => self.handle_discovery_state(fact_sheet).await,
                &AgentState::UnitTesting => self.handle_unit_testing_state(fact_sheet).await,
                _ => self.attributes.update_state(AgentState::Finished),
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_solution_architect() {
        let mut agent = AgentSolutionArchitect::new();
        let mut fact_sheet = FactSheet {
            project_description: "Build a fullstack website with user login and logout that shows latest Forex prices".to_string(),
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,
        };

        agent
            .execute(&mut fact_sheet)
            .await
            .expect("Unable to execute Solutions Architect Agent");
        assert!(fact_sheet.project_scope.is_some());
        assert!(fact_sheet.external_urls.is_some());

        dbg!(agent, fact_sheet);
    }
}
