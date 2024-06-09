use crate::ai_functions::frontend::{
    print_code_bugs_resolution, print_recommended_site_main_colours, print_recommended_site_pages,
    print_recommended_site_pages_with_apis,
};
use crate::helpers::command_line::PrintCommand;
use crate::helpers::utils::{
    ai_task_request, ai_task_request_decoded, read_frontend_code_contents, save_frontend_code,
    WEB_APP_PROJECT_PATH, WEB_SERVER_PROJECT_PATH,
};
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agent_basic::basic_traits::BasicTraits;
use crate::models::agents::agent_frontend_comp::BuildComponent;
use crate::models::agents::agent_trait::{FactSheet, SpecialFunctions};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Stdio};
use strum::IntoEnumIterator;

// To define what stage the frontend developer is at
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FrontendBuildMode {
    Infrastructure,
    PageComponents,
    Completion,
}

// For decoding the serde_json api routes for a given page
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct APIAssignment {
    pub api_route: String,
    pub method: String,
    pub route_type: String,
}

// Used for creating a type to be used for decoding shorthand
type PageRoutes = HashMap<String, Vec<APIAssignment>>;

// Used for decoding page names and suggested content
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageAPIAssign {
    pub page: Vec<APIAssignment>,
}

// Used for decoding page names and suggested content
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SitePages {
    pub page_name: String,
    pub suggested_content_sections: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DesignBuildSheet {
    pub pages: Option<Vec<String>>,
    pub pages_descriptons: Option<Vec<SitePages>>,
    pub api_assignments: Option<PageRoutes>,
    pub brand_colours: Option<Vec<String>>,
    pub build_mode: FrontendBuildMode,
}

// Solution Architect
#[derive(Debug)]
pub struct AgentFrontendDeveloper {
    pub attributes: BasicAgent,
    pub buildsheet: DesignBuildSheet,
    pub bug_count: u8,
    pub operation_focus: BuildComponent,
}

impl AgentFrontendDeveloper {
    pub fn new() -> Self {
        // Define attributes
        let attributes: BasicAgent = BasicAgent {
            objective: "Develops frontned code for website".to_string(),
            position: "Frontend Developer".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        // Define Buildsheet
        let buildsheet: DesignBuildSheet = DesignBuildSheet {
            pages: None,
            pages_descriptons: None,
            api_assignments: None,
            brand_colours: None,
            build_mode: FrontendBuildMode::Infrastructure,
        };

        // Return Self
        Self {
            attributes,
            buildsheet,
            bug_count: 0,
            operation_focus: BuildComponent::Logo,
        }
    }

    // Confirms what stage the Frontend Agent is in
    fn confirm_stage(&self) {
        match self.buildsheet.build_mode {
            FrontendBuildMode::Infrastructure => println!("[Working on Frontend Infrastructure]"),
            FrontendBuildMode::PageComponents => println!("[Working on Frontend Page Components]"),
            FrontendBuildMode::Completion => println!("[Working on Frontend Completion Items]"),
        }
    }

    // Get pages and page context from description and backend code
    async fn get_page_context(&mut self, project_description: &String) {
        // Extract backend code
        let path: String = format!("{}/src/main.rs", WEB_SERVER_PROJECT_PATH);
        let backend_code: String =
            fs::read_to_string(path).expect("Something went wrong reading the file");

        // Structure Message
        let msg_context: String = format!(
            "PROJECT_DESCRIPTION: {:?}, CODE_LOGIC: {:?}",
            project_description, backend_code
        );

        // Call AI
        let ai_response: Vec<SitePages> = ai_task_request_decoded::<Vec<SitePages>>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_recommended_site_pages),
            print_recommended_site_pages,
        )
        .await;

        // Extract pages
        let pages: Vec<String> = ai_response
            .iter()
            .filter_map(|item| Some(item.page_name.clone()))
            .collect();

        // Assign pages to buildsheet
        self.buildsheet.pages = Some(pages.clone());
        self.buildsheet.pages_descriptons = Some(ai_response);
    }

    // Assign API Routes to pages
    async fn assign_api_routes(
        &mut self,
        project_description: &String,
        external_api_urls: &Option<Vec<String>>,
    ) {
        // Extract internal API schema
        let path: String = format!("{}/api_endpoints.json", WEB_SERVER_PROJECT_PATH);
        let internal_api_endpoints: String =
            fs::read_to_string(path).expect("Something went wrong reading the file");

        // Extract external API endpoints
        let external_api_endpoints: String = match external_api_urls {
            Some(endpoints) => format!("{:?}", endpoints),
            None => String::from(""),
        };

        // Structure message for api route assignment
        let msg_context: String = format!(
            "WEBSITE SPECIFICATION: {{
      PROJECT_DESCRIPTION: {},
      PAGES: {:?},
      INTERNAL_API_ROUTES: {},
      EXTERNAL_API_ROUTES: {} 
    }}",
            project_description,
            self.buildsheet.pages,
            internal_api_endpoints,
            external_api_endpoints
        );

        // Call AI
        let ai_response: PageRoutes = ai_task_request_decoded::<PageRoutes>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_recommended_site_pages_with_apis),
            print_recommended_site_pages_with_apis,
        )
        .await;

        // Add API assignments to buildsheet
        self.buildsheet.api_assignments = Some(ai_response);
    }

    // Define Brand Colours
    async fn define_brand_colours(&mut self, project_description: &String) {
        // Structure message
        let msg_context: String = format!(
            "PROJECT_DESCRIPTION: {}, WEBSITE_CONTENT: {:?}",
            project_description, self.buildsheet.pages_descriptons
        );

        // Call AI
        let ai_response: Vec<String> = ai_task_request_decoded::<Vec<String>>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_recommended_site_main_colours),
            print_recommended_site_main_colours,
        )
        .await;

        // Add decoded brand colours
        self.buildsheet.brand_colours = Some(ai_response);
    }

    // Fix buggy component code
    async fn run_code_correction(&self, file_path: String, error_code: String) {
        // Initialize
        PrintCommand::UnitTest
            .print_agent_message(self.attributes.position.as_str(), "Fixing component bugs");
        let buggy_code: String = read_frontend_code_contents(&file_path);

        // Structure message
        let msg_context: String = format!(
            "ORIGINAL_CODE: {}, ERROR_MESSAGE: {:?}",
            buggy_code, error_code
        );

        // Retrieve AI Reponse
        let ai_response: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_code_bugs_resolution),
            print_code_bugs_resolution,
        )
        .await;

        // Save corrected code
        save_frontend_code(&file_path, &ai_response);
    }

    // Frontend component test
    async fn perform_component_test(&mut self) -> Result<(), String> {
        let test_statement = format!("Testing Component: {}", self.operation_focus.name());
        PrintCommand::UnitTest
            .print_agent_message(self.attributes.position.as_str(), test_statement.as_str());
        let build_frontend_server: std::process::Output = Command::new("yarn")
            .arg("build")
            .current_dir(WEB_APP_PROJECT_PATH)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to run component test");

        // Determine if build errors
        if build_frontend_server.status.success() {
            PrintCommand::UnitTest.print_agent_message(
                self.attributes.position.as_str(),
                "Component build test successful",
            );
            self.bug_count = 0;
            return Ok(());

        // Handle Build error
        } else {
            let error_arr: Vec<u8> = build_frontend_server.stderr;
            let error_str: String = String::from_utf8(error_arr).unwrap();

            // Check and return error
            self.bug_count += 1;
            if self.bug_count >= 2 {
                PrintCommand::Issue.print_agent_message(
                    self.attributes.position.as_str(),
                    "Too many code failures",
                );
                PrintCommand::Issue.print_agent_message(
                    self.attributes.position.as_str(),
                    "Remember: check frontend builds before retrying",
                );
                panic!(
                    "Too many code failed attempts for {}",
                    self.operation_focus.name()
                );
            } else {
                return Err(error_str);
            }
        }
    }
}

#[async_trait]
impl SpecialFunctions for AgentFrontendDeveloper {
    fn get_agent_position(&mut self) -> String {
        return self.attributes.get_position().clone();
    }

    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract required project factsheet items
        let project_description: &String = &factsheet.project_description;
        let external_api_urls: &Option<Vec<String>> = &factsheet.external_urls;

        // Continue until finished
        // !!! WARNING !!!
        while self.attributes.state != AgentState::Finished {
            // Execute logic based on Agent State
            match &self.attributes.state {
                // Get pages, api assignments and branding
                AgentState::Discovery => {
                    // Confirm Stage
                    self.confirm_stage();

                    // Get pages and page context
                    self.get_page_context(&project_description).await;

                    // Assign API routes to pages
                    self.assign_api_routes(&project_description, &external_api_urls)
                        .await;

                    // Define Brand Colours
                    self.define_brand_colours(&project_description).await;

                    // Proceed to Working status
                    self.attributes.state = AgentState::Working;
                    continue;
                }

                // Get pages, api assignments and branding
                AgentState::Working => {
                    // Loop through components
                    for component in BuildComponent::iter() {
                        // !!!! REMOVE ONLY FOR TESTING !!!
                        if component != BuildComponent::PageContent1 {
                            continue;
                        }
                        if component == BuildComponent::PageContent2 {
                            break;
                        }

                        // Update current operation focus to component
                        self.operation_focus = component.clone();
                        component
                            .create_component(&self, &project_description)
                            .await;

                        // Unit test component
                        let test_res: Result<(), String> = self.perform_component_test().await;
                        match test_res {
                            // Continue to next component
                            Ok(()) => continue,

                            // Fix bugs for current component
                            Err(err_str) => {
                                let file_path: String = self.operation_focus.filepath();
                                self.run_code_correction(file_path, err_str).await;

                                // Perform one more test
                                let _ = self.perform_component_test().await;
                                continue;
                            }
                        }
                    }

                    // Complete
                    self.attributes.state = AgentState::Finished;
                }

                // Ensure all cases are covered
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn develops_context_and_branding() {
        // Create agent instance and site purpose
        let mut agent: AgentFrontendDeveloper = AgentFrontendDeveloper::new();

        // Initialze Factsheet
        let mut factsheet: FactSheet = serde_json::from_str("{\"project_description\":\"Build a todo app for a fitness tracking goal\",\"project_scope\":{\"is_crud_required\":true,\"is_user_login_and_logout\":true,\"is_external_urls_required\":true},\"external_urls\":[\"https://api.exchangeratesapi.io/latest\"],\"backend_code\":null,\"frontend_code\":null,\"json_db_schema\":null}").unwrap();

        // Execute running agent
        agent
            .execute(&mut factsheet)
            .await
            .expect("Unable to execute running agent");
        dbg!(agent);
    }

    #[tokio::test]
    async fn works_on_shared_components() {
        // Create agent instance and site purpose
        let mut agent: AgentFrontendDeveloper = AgentFrontendDeveloper::new();
        agent.attributes.state = AgentState::Working;
        agent.buildsheet.pages = Some(vec!["home_page".to_string(), "about_page".to_string()]);

        // Initialze Factsheet
        let mut factsheet: FactSheet = serde_json::from_str("{\"project_description\":\"Build a todo app for a fitness tracking goal\",\"project_scope\":{\"is_crud_required\":true,\"is_user_login_and_logout\":true,\"is_external_urls_required\":true},\"external_urls\":[\"https://api.exchangeratesapi.io/latest\"],\"backend_code\":null,\"frontend_code\":null,\"json_db_schema\":null}").unwrap();

        // Execute running agent
        agent
            .execute(&mut factsheet)
            .await
            .expect("Unable to execute running agent");
        // dbg!(agent);
    }

    #[tokio::test]
    async fn works_on_final_pages() {
        // Create agent instance and site purpose
        let mut agent: AgentFrontendDeveloper = AgentFrontendDeveloper::new();
        let factsheet_str: &str = "{\"project_description\":\"build a website that fetches and tracks fitness progress with timezone information\",\"project_scope\":{\"is_crud_required\":true,\"is_user_login_and_logout\":true,\"is_external_urls_required\":true},\"external_urls\":[\"https://ipapi.co/json\",\"https://wger.de/api/v2/\"],\"backend_code\":\"use actix_cors::Cors;\\nuse actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};\\nuse serde::{Deserialize, Serialize};\\nuse std::sync::Mutex;\\nuse std::collections::HashMap;\\nuse std::fs;\\nuse std::io::Write;\\nuse reqwest::Client as HttpClient;\\nuse async_trait::async_trait;\\n\\n#[derive(Serialize, Deserialize, Debug, Clone)]\\npub struct FitnessProgress {\\n    pub id: u64,\\n    pub user_id: u64,\\n    pub progress_data: String,\\n    pub timezone: String,\\n}\\n\\n#[derive(Serialize, Deserialize, Debug, Clone)]\\npub struct User {\\n    pub id: u64,\\n    pub username: String,\\n    pub password: String,\\n}\\n\\n#[derive(Serialize, Deserialize, Debug, Clone)]\\nstruct Database {\\n    fitness_progresses: HashMap<u64, FitnessProgress>,\\n    users: HashMap<u64, User>,\\n}\\n\\nimpl Database {\\n    fn new() -> Self {\\n        Self {\\n            fitness_progresses: HashMap::new(),\\n            users: HashMap::new(),\\n        }\\n    }\\n\\n    // FITNESS_PROGRESS CRUD OPERATIONS\\n    fn insert_progress(&mut self, progress: FitnessProgress) {\\n        self.fitness_progresses.insert(progress.id, progress);\\n    }\\n\\n    fn get_progress(&self, id: &u64) -> Option<&FitnessProgress> {\\n        self.fitness_progresses.get(id)\\n    }\\n\\n    fn get_all_progresses(&self) -> Vec<&FitnessProgress> {\\n        self.fitness_progresses.values().collect()\\n    }\\n\\n    fn delete_progress(&mut self, id: &u64) {\\n        self.fitness_progresses.remove(id);\\n    }\\n\\n    fn update_progress(&mut self, progress: FitnessProgress) {\\n        self.fitness_progresses.insert(progress.id, progress);\\n    }\\n\\n    // USER DATA RELATED OPERATIONS\\n    fn insert_user(&mut self, user: User) {\\n        self.users.insert(user.id, user);\\n    }\\n\\n    fn get_user_by_name(&self, username: &str) -> Option<&User> {\\n        self.users.values().find(|u| u.username == username)\\n    }\\n\\n    // DATABASE SAVING\\n    fn save_to_file(&self) -> std::io::Result<()> {\\n        let data = serde_json::to_string(&self)?;\\n        let mut file = fs::File::create(\\\"database.json\\\")?;\\n        file.write_all(data.as_bytes())?;\\n        Ok(())\\n    }\\n\\n    fn load_from_file() -> std::io::Result<Self> {\\n        let file_content = fs::read_to_string(\\\"database.json\\\")?;\\n        let db: Database = serde_json::from_str(&file_content)?;\\n        Ok(db)\\n    }\\n}\\n\\nstruct AppState {\\n    db: Mutex<Database>,\\n    http_client: HttpClient,\\n}\\n\\n#[async_trait]\\ntrait ExternalDataFetcher {\\n    async fn fetch_external_data(&self, url: &str) -> Result<String, reqwest::Error>;\\n}\\n\\n#[async_trait]\\nimpl ExternalDataFetcher for AppState {\\n    async fn fetch_external_data(&self, url: &str) -> Result<String, reqwest::Error> {\\n        let response = self.http_client.get(url).send().await?;\\n        let content = response.text().await?;\\n        Ok(content)\\n    }\\n}\\n\\nasync fn create_progress(\\n    app_state: web::Data<AppState>,\\n    progress: web::Json<FitnessProgress>,\\n) -> impl Responder {\\n    let mut db = app_state.db.lock().unwrap();\\n    db.insert_progress(progress.into_inner());\\n    let _ = db.save_to_file();\\n    HttpResponse::Ok().finish()\\n}\\n\\nasync fn read_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {\\n    let db = app_state.db.lock().unwrap();\\n    match db.get_progress(&id.into_inner()) {\\n        Some(progress) => HttpResponse::Ok().json(progress),\\n        None => HttpResponse::NotFound().finish(),\\n    }\\n}\\n\\nasync fn read_all_progresses(app_state: web::Data<AppState>) -> impl Responder {\\n    let db = app_state.db.lock().unwrap();\\n    let progresses = db.get_all_progresses();\\n    HttpResponse::Ok().json(progresses)\\n}\\n\\nasync fn update_progress(\\n    app_state: web::Data<AppState>,\\n    progress: web::Json<FitnessProgress>,\\n) -> impl Responder {\\n    let mut db = app_state.db.lock().unwrap();\\n    db.update_progress(progress.into_inner());\\n    let _ = db.save_to_file();\\n    HttpResponse::Ok().finish()\\n}\\n\\nasync fn delete_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {\\n    let mut db = app_state.db.lock().unwrap();\\n    db.delete_progress(&id.into_inner());\\n    let _ = db.save_to_file();\\n    HttpResponse::Ok().finish()\\n}\\n\\nasync fn register(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {\\n    let mut db = app_state.db.lock().unwrap();\\n    db.insert_user(user.into_inner());\\n    let _ = db.save_to_file();\\n    HttpResponse::Ok().finish()\\n}\\n\\nasync fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {\\n    let db = app_state.db.lock().unwrap();\\n\\n    match db.get_user_by_name(&user.username) {\\n        Some(stored_user) if stored_user.password == user.password => {\\n            HttpResponse::Ok().body(\\\"Logged in!\\\")\\n        }\\n        _ => HttpResponse::BadRequest().body(\\\"Invalid username or password\\\"),\\n    }\\n}\\n\\n#[actix_web::main]\\nasync fn main() -> std::io::Result<()> {\\n    let db = match Database::load_from_file() {\\n        Ok(db) => db,\\n        Err(_) => Database::new(),\\n    };\\n\\n    let data = web::Data::new(AppState {\\n        db: Mutex::new(db),\\n        http_client: HttpClient::new(),\\n    });\\n\\n    HttpServer::new(move || {\\n        App::new()\\n            .wrap(\\n                Cors::permissive()\\n                    .allowed_origin_fn(|origin, _req_head| {\\n                        origin.as_bytes().starts_with(b\\\"http://localhost:\\\") || origin == \\\"null\\\"\\n                    })\\n                    .allowed_methods(vec![\\\"GET\\\", \\\"POST\\\", \\\"PUT\\\", \\\"DELETE\\\"])\\n                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])\\n                    .allowed_header(header::CONTENT_TYPE)\\n                    .supports_credentials()\\n                    .max_age(3600),\\n            )\\n            .app_data(data.clone())\\n            .route(\\\"/progress\\\", web::post().to(create_progress))\\n            .route(\\\"/progress\\\", web::get().to(read_all_progresses))\\n            .route(\\\"/progress/{id}\\\", web::get().to(read_progress))\\n            .route(\\\"/progress/{id}\\\", web::put().to(update_progress))\\n            .route(\\\"/progress/{id}\\\", web::delete().to(delete_progress))\\n            .route(\\\"/register\\\", web::post().to(register))\\n            .route(\\\"/login\\\", web::post().to(login))\\n    })\\n    .bind(\\\"127.0.0.1:8080\\\")?\\n    .run()\\n    .await\\n}\",\"api_endpoint_schema\":[{\"is_route_dynamic\":\"false\",\"method\":\"get\",\"request_body\":\"None\",\"response\":\"Array\",\"route\":\"/progress\"}]}";
        let buildsheet_str: &str = "{\"pages\":[\"home_page\",\"progress_dashboard\"],\"pages_descriptons\":[{\"page_name\":\"home_page\",\"suggested_content_sections\":{\"banner_section\":\"Catchy title and subtitle showcasing the fitness progress tracking features\",\"call_to_action_section\":\"Encourage users to sign up and start tracking their fitness progress\",\"features_section\":\"Display key features of the website with icons and short descriptions\"}},{\"page_name\":\"progress_dashboard\",\"suggested_content_sections\":{\"add_progress_section\":\"Provide a form for the user to input new fitness progress data\",\"fitness_progress_section\":\"Display a visual representation of the user's fitness progress over time\",\"user_info_section\":\"Display user's name, timezone info and greetings based on the time of the day\"}}],\"api_assignments\":{\"home_page\":[{\"api_route\":\"/register\",\"method\":\"post\",\"route_type\":\"internal\"},{\"api_route\":\"/login\",\"method\":\"post\",\"route_type\":\"internal\"},{\"api_route\":\"https://ipapi.co/json\",\"method\":\"get\",\"route_type\":\"external\"}],\"progress_dashboard\":[{\"api_route\":\"/progress\",\"method\":\"post\",\"route_type\":\"internal\"},{\"api_route\":\"/progress\",\"method\":\"get\",\"route_type\":\"internal\"},{\"api_route\":\"/progress/{id}\",\"method\":\"get\",\"route_type\":\"internal\"},{\"api_route\":\"/progress/{id}\",\"method\":\"put\",\"route_type\":\"internal\"},{\"api_route\":\"/progress/{id}\",\"method\":\"delete\",\"route_type\":\"internal\"},{\"api_route\":\"https://wger.de/api/v2/\",\"method\":\"get\",\"route_type\":\"external\"}]},\"brand_colours\":[\"#32a852\",\"#0fa0d1\",\"#d10fcb\"],\"build_mode\":\"Infrastructure\"}";
        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();
        let buildsheet: DesignBuildSheet = serde_json::from_str(buildsheet_str).unwrap();
        agent.attributes.state = AgentState::Working;

        agent.buildsheet = buildsheet;
        agent
            .execute(&mut factsheet)
            .await
            .expect("Unable to execute running agent");
        agent.attributes.state = AgentState::Working;
    }
}
