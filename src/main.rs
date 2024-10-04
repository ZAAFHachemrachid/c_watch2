use clap::Parser;
use colored::*;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Child;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

#[derive(Parser)]
struct Args {
    /// Path to the C file to watch
    file_path: String,

    /// Clear the screen before recompiling
    #[arg(short = 'c', long = "clear")]
    clear_screen: bool,

    /// Time to wait if program returns 5 (in seconds)
    #[arg(short = 't', long = "time", default_value_t = 5)]
    wait_time: u64,

    /// Delay between recompilation in seconds
    #[arg(short = 'd', long = "delay", default_value_t = 0)]
    delay: u64,
}

fn compile_c_file(file_path: &str) -> std::io::Result<bool> {
    let output = Command::new("gcc")
        .arg(file_path)
        .arg("-o")
        .arg("output")
        .output()?;

    if output.status.success() {
        println!("{}", "Compilation successful.".green());
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", format!("Compilation failed:\n{}", stderr).red());
        Ok(false)
    }
}

fn sleep_seconds(seconds: u64) {
    sleep(Duration::from_secs(seconds));
}

fn run_binary(
    child: &Arc<Mutex<Option<Child>>>,
    wait_time: u64,
    force_recompile: Arc<Mutex<bool>>, // New flag to force recompilation
) -> std::io::Result<()> {
    {
        // Kill existing child if running
        let mut child_lock = child.lock().unwrap();
        if let Some(existing_child) = child_lock.as_mut() {
            let _ = existing_child.kill();
            let _ = existing_child.wait();
        }
    }

    let child_clone = Arc::clone(child);
    let force_recompile_clone = Arc::clone(&force_recompile); // Clone the flag
    thread::spawn(move || {
        let mut new_child = Command::new("./output")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to run the compiled binary");

        let exit_status = new_child.wait().expect("Failed to wait on child");

        // If exit code is 5, wait for the specified duration
        if let Some(5) = exit_status.code() {
            println!("{}", format!("================").bright_green().bold());
            println!(
                " {}",
                format!("Code run successful ,Edit file for recompilation")
                    .yellow()
                    .bold()
                    .italic()
            );

            // After the wait, set the flag to force a recompilation
            let mut force_recompile_lock = force_recompile_clone.lock().unwrap();
            *force_recompile_lock = true;
        }

        // Update the child process in the mutex
        let mut child_lock = child_clone.lock().unwrap();
        *child_lock = None;
    });

    Ok(())
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg("cls").status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let path = Path::new(&args.file_path);
    if !path.exists() {
        eprintln!(
            "{}",
            format!("Error: file '{}' does not exist.", args.file_path).red()
        );
        std::process::exit(1);
    }

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    compile_c_file(&args.file_path)?;

    println!(
        "{}",
        format!("Watching file: {}", args.file_path)
            .cyan()
            .bold()
            .underline()
    );

    let child_process = Arc::new(Mutex::new(None));
    let force_recompile = Arc::new(Mutex::new(false)); // Flag to force recompilation
    let mut last_event_time = std::time::Instant::now();

    loop {
        // Check if recompilation should be forced due to exit code 5
        {
            let mut force_recompile_lock = force_recompile.lock().unwrap();
            if *force_recompile_lock {
                *force_recompile_lock = false; // Reset the flag after recompilation is forced

                if args.clear_screen {
                    clear_screen();
                }

                println!(
                    "{}",
                    "Forcing recompilation after wait...".bright_blue().bold()
                );
                println!("{}", format!("================").bright_green().bold());

                if compile_c_file(&args.file_path).unwrap() {
                    println!("{}", "Running the program:".green());
                    run_binary(&child_process, args.wait_time, Arc::clone(&force_recompile))?;
                }
                continue; // Skip to the next iteration after forcing recompilation
            }
        }

        // Process regular file change events
        match rx.recv() {
            Ok(Ok(event)) => {
                if let EventKind::Modify(_) = event.kind {
                    if last_event_time.elapsed().as_secs() < 1 {
                        continue;
                    }
                    last_event_time = std::time::Instant::now();

                    if args.delay > 0 {
                        sleep_seconds(args.delay);
                    }

                    if args.clear_screen {
                        clear_screen();
                    }

                    println!("{}", "File changed, recompiling...".bright_blue().bold());
                    println!("{}", format!("================").bright_green().bold());

                    if compile_c_file(&args.file_path).unwrap() {
                        println!("{}", "Running the program:".green());
                        run_binary(&child_process, args.wait_time, Arc::clone(&force_recompile))?;
                    } else {
                        println!("{}", "Compilation failed, not running the program.".red());
                    }
                }
            }
            Ok(Err(e)) => eprintln!("{}", format!("Watch error: {:?}", e).red()),
            Err(e) => eprintln!("{}", format!("Channel error: {:?}", e).red()),
        }
    }
}
