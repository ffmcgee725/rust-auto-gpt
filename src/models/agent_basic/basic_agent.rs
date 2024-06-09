use crate::models::agent_basic::basic_traits::BasicTraits;
use crate::models::general::llm::Message;

#[derive(Debug, PartialEq)]
pub enum AgentState {
    Discovery,
    Finished,
    UnitTesting,
    Working,
}

#[derive(Debug)]
pub struct BasicAgent {
    pub objective: String,
    pub position: String,
    pub state: AgentState,
    pub memory: Vec<Message>,
}

impl BasicTraits for BasicAgent {
    fn new(objective: String, position: String) -> Self {
        Self {
            objective,
            position,
            state: AgentState::Discovery,
            memory: Vec::new(),
        }
    }

    fn update_state(&mut self, new_state: AgentState) {
        self.state = new_state;
    }

    fn get_objective(&mut self) -> &String {
        return &self.objective;
    }

    fn get_position(&mut self) -> &String {
        return &self.position;
    }

    fn get_state(&mut self) -> &AgentState {
        return &self.state;
    }

    fn get_memory(&mut self) -> &Vec<Message> {
        return &self.memory;
    }
}
