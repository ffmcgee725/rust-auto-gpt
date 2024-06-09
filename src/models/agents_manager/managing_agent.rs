use crate::{
    ai_functions::managing::convert_user_input_to_goal,
    helpers::utils::ai_task_request,
    models::{
        agent_basic::{
            basic_agent::{AgentState, BasicAgent},
            basic_traits::BasicTraits,
        },
        agents::{
            agent_architect::AgentSolutionArchitect,
            agent_backend::AgentBackendDeveloper,
            agent_trait::{FactSheet, SpecialFunctions},
        },
    },
};

#[derive(Debug)]
pub struct ManagingAgent {
    attributes: BasicAgent,
    fact_sheet: FactSheet,
    agents: Vec<Box<dyn SpecialFunctions>>,
}

impl ManagingAgent {
    pub async fn new(user_request: String) -> Result<Self, Box<dyn std::error::Error>> {
        let position: String = "Project Manager".to_string();
        let attributes: BasicAgent = BasicAgent {
            objective: "Manage agents who are building an excellent website for a user."
                .to_string(),
            position: position.clone(),
            state: AgentState::Discovery,
            memory: Vec::new(),
        };

        let project_description: String = ai_task_request(
            user_request,
            &position,
            get_function_string!(convert_user_input_to_goal),
            convert_user_input_to_goal,
        )
        .await;

        let agents: Vec<Box<dyn SpecialFunctions>> = vec![];

        let fact_sheet: FactSheet = FactSheet {
            project_description,
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,
        };

        return Ok(Self {
            attributes,
            fact_sheet,
            agents,
        });
    }

    pub async fn execute_project(&mut self) {
        self.create_agents();
        for agent in &mut self.agents {
            let position: String = agent.get_agent_position();
            let error_message: String = format!("{} failed to execute", position);

            if let Err(e) = agent.execute(&mut self.fact_sheet).await {
                panic!("{} -- Error: {}", error_message, e);
            }
        }
    }

    fn create_agents(&mut self) {
        self.add_agent(Box::new(AgentSolutionArchitect::new()));
        self.add_agent(Box::new(AgentBackendDeveloper::new()));
    }

    fn add_agent(&mut self, agent: Box<dyn SpecialFunctions>) {
        self.agents.push(agent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_managing_agent() {
        let user_request: &str =
            "Build me a website that randomly shows cool pictures of cats. I should query an endpoint, and retrieve a random image from a cat fetched from an external API!";
        let mut agent: ManagingAgent = ManagingAgent::new(user_request.to_string())
            .await
            .expect("Error creating Managing Agent");

        agent.execute_project().await;

        assert!(agent.fact_sheet.project_scope.is_some());
        assert!(agent.fact_sheet.external_urls.is_some());

        dbg!(agent.fact_sheet);
    }
}
