use chrono::Local;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::format;
use std::fs::{self, File};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Experiment {
    id: String,          // UUID
    name: String,        // name of the experiment
    script_path: String, //path of the script
    args: Vec<String>,   // arguements
    timestamp: String,
    result: Option<serde_json::Value>, // save the result of the experiment
}

fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("register") => register_experiment(),
        Some("run") => {
            if let Some(id) = args.get(2) {
                run_experiment_by_id(id);
            } else {
                run_experiment();
            }
        }
        Some("list") => list_experiments(),
        Some("delete") => {
            if let Some(id) = args.get(2) {
                delete_experiment(id);
            } else {
                println!("Usage: cargo run -- delete <experiment_id>");
            }
        }
        _ => {
            println!("Usage");
            println!(" register     # Register new experiment");
            println!(" run          # run latest experiment");
            println!(" run <id>     # run specific experiment");
            println!(" list         # Register new experiment");
            println!(" delete <id>  # Register new experiment");
        }
    }
}
fn run_experiment_by_id(id: &str) {}

fn register_experiment() {
    println!("register_experiment");

    // create an Uuid
    let id = Uuid::new_v4().to_string();

    // current time
    let timestamp = Local::now().to_rfc3339();

    let dir_path = format!("experiments/{}", id);
    fs::create_dir_all(&dir_path).expect("Failed to create directory");

    // copy the python code
    let original_script = "train.py";
    let copied_script_path = format!("{}/train.py", dir_path);
    fs::copy(original_script, &copied_script_path).expect("Failed to copy python code");

    let experiment = Experiment {
        id: id.clone(),
        name: "example".to_string(),
        script_path: "train.py".to_string(),
        args: vec!["--lr=0.01".to_string(), "--batch=32".to_string()],
        timestamp,
        result: None,
    };

    let file_path = format!("{}/meta.json", dir_path);
    let file = File::create(file_path).expect("Failed to create file");
    serde_json::to_writer_pretty(file, &experiment).expect("Failed to save JSON");

    println!("Experiment register complete");
}

fn find_latest_experiment() -> Option<String> {
    let mut entries: Vec<_> = std::fs::read_dir("experiments")
        .ok()? // 실패시 None return
        .filter_map(|e| e.ok()) // 오류 없는 항목만 추출
        .collect(); // Vec<DirEntry> 로 모음
    // 수정시간으로 정렬
    entries.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());

    // 최신 항목 pop 함
    // 폴더 이름을 String 으로 변환
    entries
        .pop()
        .map(|e| e.file_name().to_string_lossy().into_owned())
}

fn run_experiment() {
    println!("run_experiment");
    // find the latest experiment
    let id = match find_latest_experiment() {
        Some(id) => id,
        None => {
            println!("There are none");
            return;
        }
    };
    // configure experiment directory and meta.json path
    let dir = format!("experiments/{}", id);
    let meta_path = format!("{}/meta.json", dir);

    // read meta.json
    let file = std::fs::File::open(&meta_path).expect("Failed to open meta.json");
    let reader = std::io::BufReader::new(file);
    let mut experiment: Experiment =
        serde_json::from_reader(reader).expect("Failed to parse meta.json");

    // python script path
    let script_path = format!("{}/{}", dir, experiment.script_path);

    // open the log file
    let log_path = format!("{}/log.txt", dir);
    let mut log_file = std::fs::File::create(&log_path).expect("Faild to create log file");

    // run sub porcess
    use std::io::Write;
    use std::process::{Command, Stdio};

    let output = Command::new("python3")
        .arg("train.py")
        .args(&experiment.args)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run python");

    writeln!(
        log_file,
        "stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    )
    .unwrap();
    writeln!(
        log_file,
        "\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    )
    .unwrap();

    println!("Experiment done : log saved {}", log_path);

    let metrics_path = format!("{}/metrics.json", dir);
    if let Ok(metrics_file) = std::fs::File::open(&metrics_path) {
        let metrics: serde_json::Value =
            serde_json::from_reader(metrics_file).expect("Failed parse metrics.json");
        experiment.result = Some(metrics);

        println!("Result has been applied {}", metrics_path);
    } else {
        println!("There is no metrics.json omitting the result");
    }
}

// list the experiment
fn list_experiments() {
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;

    println!("list_experiment");

    // read the directory
    let entries = match fs::read_dir("experiments") {
        Ok(e) => e,
        Err(_) => {
            println!("(No experiments directory found");
            return;
        }
    };

    let mut found = false;

    // get the meta data
    for entry in entries.filter_map(|e| e.ok()) {
        let meta_path = entry.path().join("meta.json");

        if !meta_path.exists() {
            continue;
        }
        let file = match File::open(&meta_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let reader = BufReader::new(file);
        let experiment: Experiment = match serde_json::from_reader(reader) {
            Ok(e) => e,
            Err(_) => continue,
        };

        found = true;

        println!(
            "\n ID: {}\n Name: {}\n Time: {} \n Result: {}",
            experiment.id,
            experiment.name,
            experiment.timestamp,
            match &experiment.result {
                Some(val) => val.to_string(),
                None => "None".to_string(),
            }
        );
    }

    if !found {
        println!("(No experiments found");
    }
}

// delete the experiment
fn delete_experiment(id: &str) {
    let path = format!("experiments/{}", id);

    if std::path::Path::new(&path).exists() {
        match std::fs::remove_dir_all(&path) {
            Ok(_) => println!("Deleted experiment: {}", id),
            Err(e) => println!("Failed to delete {}: {}", id, e),
        }
    } else {
        println!("No such experiment : {}", id)
    }
}
