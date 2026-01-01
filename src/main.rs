mod graph;
mod llm;
mod agent;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use dotenv::dotenv;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

use agent::AgenticMemory;
use llm::{LLMClient, LLMProvider};

/// Agentic Memory - AI Assistant with Context Graph powered by GraphLite
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the GraphLite database
    #[arg(short, long, default_value = "./data/memory.db")]
    db_path: String,

    /// Database admin username
    #[arg(short, long, default_value = "admin")]
    user: String,

    /// Database admin password
    #[arg(short, long, default_value = "admin123")]
    password: String,

    /// Conversation title
    #[arg(short, long)]
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Display welcome banner
    print_banner();

    // Initialize LLM client
    let llm_client = create_llm_client()?;

    // Initialize agentic memory
    println!("{}", "Initializing agentic memory system...".cyan());
    let mut memory = AgenticMemory::new(
        &args.db_path,
        &args.user,
        &args.password,
        llm_client,
    )
    .await
    .context("Failed to initialize agentic memory")?;

    // Create database session
    let session = memory
        .session(&args.user, &args.password)
        .context("Failed to create database session")?;

    // Start a new conversation
    let conversation_title = args.title.or_else(|| Some("New Conversation".to_string()));
    let conv_id = memory
        .start_conversation(&session, conversation_title)
        .context("Failed to start conversation")?;

    println!("{}", format!("Started conversation: {}", conv_id).green());
    println!("{}", "Type your message and press Enter. Use 'exit' or 'quit' to end the conversation.\n".yellow());

    // Interactive REPL
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(&format!("{} ", "You:".bright_blue().bold()));

        match readline {
            Ok(line) => {
                let user_input = line.trim();

                // Check for exit commands
                if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
                    println!("{}", "Goodbye! Your memory has been saved.".green());
                    break;
                }

                // Skip empty messages
                if user_input.is_empty() {
                    continue;
                }

                // Add to history
                rl.add_history_entry(user_input)?;

                // Process the message
                match process_message(&memory, &session, user_input).await {
                    Ok(response) => {
                        println!("{} {}\n", "Assistant:".bright_green().bold(), response);

                        // Store assistant response
                        if let Err(e) = memory.store_assistant_message(&session, &response) {
                            eprintln!("{}", format!("Warning: Failed to store assistant message: {}", e).yellow());
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Error: {}", e).red());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "Interrupted. Use 'exit' or 'quit' to end the conversation.".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye! Your memory has been saved.".green());
                break;
            }
            Err(err) => {
                eprintln!("{}", format!("Error: {}", err).red());
                break;
            }
        }
    }

    Ok(())
}

/// Process a user message and generate a response
async fn process_message(
    memory: &AgenticMemory,
    session: &graphlite_rust_sdk::Session,
    message: &str,
) -> Result<String> {
    // Show thinking indicator
    print!("{}", "Thinking... ".cyan());
    std::io::Write::flush(&mut std::io::stdout())?;

    // Extract entities and store message
    let (msg_id, entities) = memory
        .process_user_message(session, message)
        .await
        .context("Failed to process user message")?;

    // Print extracted entities if any
    if !entities.people.is_empty()
        || !entities.topics.is_empty()
        || !entities.tasks.is_empty()
    {
        println!("\r{}", "                    \r".to_string()); // Clear thinking indicator

        let mut entity_info = Vec::new();
        if !entities.people.is_empty() {
            entity_info.push(format!("People: {}", entities.people.join(", ")));
        }
        if !entities.topics.is_empty() {
            entity_info.push(format!("Topics: {}", entities.topics.join(", ")));
        }
        if !entities.tasks.is_empty() {
            entity_info.push(format!("Tasks: {}", entities.tasks.join(", ")));
        }

        println!("{}", format!("[Extracted: {}]", entity_info.join(" | ")).dimmed());
    } else {
        println!("\r{}", "                    \r".to_string()); // Clear thinking indicator
    }

    // Generate response with context
    let response = memory
        .generate_response(session, message, &entities)
        .await
        .context("Failed to generate response")?;

    Ok(response)
}

/// Create LLM client from environment variables
fn create_llm_client() -> Result<LLMClient> {
    let provider_name = env::var("LLM_PROVIDER")
        .unwrap_or_else(|_| "anthropic".to_string())
        .to_lowercase();

    let provider = match provider_name.as_str() {
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY not set in environment")?;
            let model = env::var("LLM_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

            LLMProvider::Anthropic { api_key, model }
        }
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY not set in environment")?;
            let model = env::var("LLM_MODEL")
                .unwrap_or_else(|_| "gpt-4-turbo-preview".to_string());

            LLMProvider::OpenAI { api_key, model }
        }
        _ => {
            anyhow::bail!("Unknown LLM_PROVIDER: {}. Use 'anthropic' or 'openai'", provider_name);
        }
    };

    Ok(LLMClient::new(provider))
}

/// Print welcome banner
fn print_banner() {
    println!("\n{}", "╔══════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                          ║".bright_cyan());
    println!("{}", "║              🧠 AGENTIC MEMORY SYSTEM 🧠                 ║".bright_cyan().bold());
    println!("{}", "║                                                          ║".bright_cyan());
    println!("{}", "║        AI Assistant with Context Graph Memory            ║".bright_cyan());
    println!("{}", "║              Powered by GraphLite                        ║".bright_cyan());
    println!("{}", "║                                                          ║".bright_cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
}
