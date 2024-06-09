use super::basic_agent::AgentState;
use crate::models::general::llm::Message;

pub trait BasicTraits {
    fn new(objective: String, position: String) -> Self;
    fn update_state(&mut self, new_state: AgentState);
    fn get_objective(&mut self) -> &String;
    fn get_position(&mut self) -> &String;
    fn get_state(&mut self) -> &AgentState;
    fn get_memory(&mut self) -> &Vec<Message>;
}
