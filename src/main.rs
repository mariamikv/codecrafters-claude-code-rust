use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let model = env::var("LOCAL_MODEL").unwrap_or("anthropic/claude-haiku-4.5".to_string());
    let tools = tools();

    #[allow(unused_variables)]
    let response: Value = client
        .chat()
        .create_byot(json!({
            "messages": [
                {
                    "role": "user",
                    "content": args.prompt
                }
            ],
            "model": model,
            "tools": tools,
        }))
        .await?;

    eprintln!("Logs from your program will appear here!");

    let message = &response["choices"][0]["message"];

    if let Some(tool_calls) = message["tool_calls"].as_array() {
        if let Some(call) = tool_calls.first() {
            let function = &call["function"];
            let name = function["name"].as_str().unwrap_or_default();
            let raw_args = function["arguments"].as_str().unwrap_or("{}");
            let arguments: Value = serde_json::from_str(raw_args)?;

            match name {
                "read" | "Read" => {
                    let file_path = arguments["file_path"]
                        .as_str()
                        .ok_or("missing file_path argument")?;
                    let contents = std::fs::read_to_string(file_path)?;
                    print!("{contents}");
                }
                other => eprintln!("Unknown tool: {other}"),
            }
        }
    } else if let Some(content) = message["content"].as_str() {
        println!("{content}");
    }

    Ok(())
}

pub fn tools() -> Vec<Value> {
    let tools = vec![serde_json::from_str(include_str!("tools/read.json")).unwrap()];
    tools
}