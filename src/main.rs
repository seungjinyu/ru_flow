use chrono::Local;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};
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

fn is_process_alive(pid: u32) -> bool {
    let mut sys = System::new();
    sys.refresh_processes();
    sys.process(sysinfo::Pid::from_u32(pid)).is_some()
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
                delete_experiment_interactive();
            }
        }
        Some("logs") => {
            if let Some(id) = args.get(2) {
                show_logs(id);
            } else {
                show_logs_interactive();
            }
        }
        Some("status") => {
            show_running_experiments();
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
fn show_running_experiments() {}

fn show_logs_interactive() {
    let entries = match std::fs::read_dir("experiments") {
        Ok(e) => e,
        Err(_) => {
            println!("No experiments directory found");
            return;
        }
    };

    let mut experiments = vec![];
    for entry in entries.filter_map(|e| e.ok()) {
        let meta_path = entry.path().join("meta.json");
        if meta_path.exists() {
            if let Ok(file) = File::open(&meta_path) {
                let reader = BufReader::new(file);
                if let Ok(exp) = serde_json::from_reader::<_, Experiment>(reader) {
                    let dir_id = entry.file_name().to_string_lossy().into_owned();
                    experiments.push((dir_id, exp));
                }
            }
        }
    }

    if experiments.is_empty() {
        println!("No experiments found");
        return;
    }

    println!("Select the experiment to view logs:");
    for (i, (id, exp)) in experiments.iter().enumerate() {
        println!(
            "[{}] ID: {} | {} | {}",
            i,
            id,
            exp.name,
            exp.timestamp.split('T').next().unwrap_or("")
        );
    }

    print!("Enter number to view logs: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    match trimmed.parse::<usize>() {
        Ok(index) if index < experiments.len() => {
            let selected_id = &experiments[index].0;
            show_logs(selected_id);
        }
        _ => {
            println!("Invalid selection");
        }
    }
}
fn show_logs(id: &str) {
    let log_path = format!("experiments/{}/log.txt", id);

    if !Path::new(&log_path).exists() {
        eprintln!("No log file found for experiment {}", id);
        return;
    }
    let mut child = Command::new("tail")
        .arg("-f")
        .arg(&log_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect(":Failed to run tail");
    let _ = child.wait();
}

fn run_experiment_by_id(id: &str) {
    let dir = format!("experiments/{}", id);
    let meta_path = format!("{}/meta.json", dir);
    let lock_path = format!("{}/run.lock", dir);

    if Path::new(&lock_path).exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if is_process_alive(pid) {
                    println!("Experiment {} is now on (PID: {})", id, pid);
                    return;
                } else {
                    println!("Previous Experiment(PID: {}) is already done", pid);
                    std::fs::remove_file(&lock_path).ok();
                }
            }
        }
    }

    if !Path::new(&meta_path).exists() {
        println!("There are no experiment for that id {}", id);
        return;
    }

    // open meta.json
    let file = File::open(&meta_path).expect("Failed to open meta.json");
    let reader = BufReader::new(file);
    let experiment: Experiment =
        serde_json::from_reader(reader).expect("Failed to parse meta.json");

    // log file
    let log_path = format!("{}/log.txt", dir);
    let log_file = File::create(&log_path).expect("Failed to create log file");

    let child = Command::new("python3")
        .arg("train.py")
        .args(&experiment.args)
        .current_dir(&dir)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to run experiment");

    std::fs::write(&lock_path, child.id().to_string()).expect("Failed to writh run.lock");
    println!("Experiment {} on process PID {}", id, child.id());
}

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
    // configure experiment directory and meta.json path lock file
    let dir = format!("experiments/{}", id);
    let meta_path = format!("{}/meta.json", dir);
    let lock_path = format!("{}/run.lock", dir);

    if Path::new(&lock_path).exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if is_process_alive(pid) {
                    println!("Experiment {} is now on (PID: {})", id, pid);
                    return;
                } else {
                    println!("Previous Experiment(PID: {}) is already done", pid);
                    std::fs::remove_file(&lock_path).ok();
                }
            }
        }
    }

    if !Path::new(&meta_path).exists() {
        println!("There are no meta.json for {}", id);
        return;
    }

    // read meta.json
    let file = std::fs::File::open(&meta_path).expect("Failed to open meta.json");
    let reader = std::io::BufReader::new(file);
    let experiment: Experiment =
        serde_json::from_reader(reader).expect("Failed to parse meta.json");

    // open the log file
    let log_path = format!("{}/log.txt", dir);
    let log_file = std::fs::File::create(&log_path).expect("Faild to create log file");

    // run sub porcess
    let mut child = Command::new("python3")
        .arg("train.py")
        .args(&experiment.args)
        .current_dir(&dir)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to run python");
    let lock_path = format!("{}/run.lock", dir);
    std::fs::write(&lock_path, child.id().to_string()).expect("Faild to save run.lock ");

    println!("Experiment {} Started PID {}", id, child.id());
    let lock_path_clone = lock_path.clone();

    std::thread::spawn(move || {
        let _ = child.wait();
        match std::fs::remove_file(&lock_path_clone) {
            Ok(_) => println!("run.lock deletion completed {}", lock_path_clone),
            Err(e) => println!("run.lock deleteion failed {}", e),
        }
    });
}

// list the experiment
fn list_experiments() {
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

fn delete_experiment_interactive() {
    let entries = match std::fs::read_dir("experiments") {
        Ok(e) => e,
        Err(_) => {
            println!("No experiments directory found.");
            return;
        }
    };

    let mut experiments = vec![];
    for entry in entries.filter_map(|e| e.ok()) {
        let meta_path = entry.path().join("meta.json");
        if meta_path.exists() {
            if let Ok(file) = File::open(&meta_path) {
                let reader = std::io::BufReader::new(file);
                if let Ok(exp) = serde_json::from_reader::<_, Experiment>(reader) {
                    let dir_id = entry.file_name().to_string_lossy().into_owned();
                    experiments.push((dir_id, exp));
                }
            }
        }
    }

    if experiments.is_empty() {
        println!("No experiments to delete");
        return;
    }

    println!("Select the experiement to delete:");
    for (i, (id, exp)) in experiments.iter().enumerate() {
        let status = if exp.result.is_some() {
            "Available"
        } else {
            "NA"
        };
        println!(
            "[{}] {} ID: {} | {} | {}",
            i,
            status,
            id,
            exp.name,
            exp.timestamp.split('T').next().unwrap_or("")
        );
    }
    print!("Enter in the number to delete : ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();

    if trimmed.is_empty() {
        println!("Canceling deletion");
        return;
    }

    match trimmed.parse::<usize>() {
        Ok(index) if index < experiments.len() => {
            let id = &experiments[index].0;
            let path = format!("experiments/{}", id);
            match fs::remove_dir_all(&path) {
                Ok(_) => println!("Deletion complete {}", id),
                Err(e) => println!("Deleteion failed {}", e),
            }
        }
        _ => {
            println!("Wrong input");
        }
    };
}
