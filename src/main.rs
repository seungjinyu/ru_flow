use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Experiment {
    id: String,
    name: String,
    script_path: String,
    args: Vec<String>,
    timestamp: String,
    result: Option<serde_json::Value>,
}

fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("register") => register_experiment(),
        Some("run") => run_experiment(),
        Some("list") => list_experiments(),
        _ => {
            println!("Usage");
            println!(" register # Register new experiment");
            println!(" run      # run new experiment");
            println!(" list     # Register new experiment");
        }
    }
}

fn register_experiment() {
    println!("register_experiment");
}
fn run_experiment() {
    println!("run_experiment");
}
fn list_experiments() {
    println!("list_experiment");
}
