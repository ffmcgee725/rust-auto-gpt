#[macro_export]
macro_rules! get_function_string {
    ($func: ident) => {{
        stringify!($func)
    }};
}

#[macro_use]
mod ai_functions;
mod apis;
mod helpers;
mod models;

use helpers::command_line::get_user_input;
use models::agents_manager::managing_agent::ManagingAgent;
#[tokio::main]
async fn main() {
    let user_req: String = get_user_input("What website are we building today?");

    let mut managing_agent: ManagingAgent = ManagingAgent::new(user_req)
        .await
        .expect("Failed to start up Managing Agent");

    managing_agent.execute_project().await;
    dbg!(managing_agent);
}
