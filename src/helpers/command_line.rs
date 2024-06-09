use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

use std::io::{stdin, stdout};

#[derive(PartialEq, Debug)]
pub enum PrintCommand {
    AICall,
    UnitTest,
    Issue,
}

impl PrintCommand {
    pub fn print_agent_message(&self, agent_position: &str, agent_statement: &str) -> () {
        let mut stdout: std::io::Stdout = stdout();

        let statement_color: Color = match self {
            Self::AICall => Color::Cyan,
            Self::UnitTest => Color::Magenta,
            Self::Issue => Color::Red,
        };

        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        print!("Agent: {}: ", agent_position);

        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", agent_statement);

        stdout.execute(ResetColor).unwrap();
    }
}

pub fn get_user_input(prompt: &str) -> String {
    print_prompt_in_color(&prompt, Color::Blue);
    return read_user_input().trim().to_string();
}

pub fn confirm_safe_code() -> bool {
    let mut stdout: std::io::Stdout = stdout();
    loop {
        stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
        println!("");
        println!("WARNING: You are about to run code entirely written by AI.");
        println!("Confirm your code and confirm you wish to continue.");

        stdout.execute(ResetColor).unwrap();

        // Present options
        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        println!("[1]. All good, execute!");

        stdout.execute(SetForegroundColor(Color::DarkRed)).unwrap();
        println!("[2]. Abort!");

        stdout.execute(ResetColor).unwrap();

        let mut human_response = String::new();
        stdin()
            .read_line(&mut human_response)
            .expect("Failed to read response");
        let human_response = human_response.trim().to_lowercase();

        match human_response.as_str() {
            "1" | "ok" | "y" | "yes" => return true,
            "2" | "no" | "n" | "abort" => return false,
            _ => println!("Invalid response. Select '1' or '2'"),
        };
    }
}

fn print_prompt_in_color(prompt: &str, color: Color) -> () {
    let mut stdout: std::io::Stdout = stdout();

    stdout.execute(SetForegroundColor(color)).unwrap();
    println!("");
    println!("{}", prompt);

    stdout.execute(ResetColor).unwrap();
}

fn read_user_input() -> String {
    let mut user_response: String = String::new();
    stdin()
        .read_line(&mut user_response)
        .expect("Failed to read response");
    return user_response;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_print_agent_message() {
        PrintCommand::AICall.print_agent_message(
            "Managing Agent",
            "Testing testing, processing something in cyan",
        );
        PrintCommand::Issue.print_agent_message(
            "Managing Agent",
            "Testing testing, processing something in red",
        );
        PrintCommand::UnitTest.print_agent_message(
            "Managing Agent",
            "Testing testing, processing something in magenta",
        );
    }
}
