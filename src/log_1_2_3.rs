use clap::Parser;
use colored::*; // Import the colored crate
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration; // Import the fs module for file operations

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
    // Compile the C file and return the result
    let output = Command::new("gcc")
        .arg(file_path)
        .arg("-o")
        .arg("output") // Specify the output binary name
        .output()?; // Capture the output

    // Check if the compilation was successful
    if output.status.success() {
        println!("{}", "Compilation successful.".green());
        Ok(true)
    } else {
        // Print compilation error
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", format!("Compilation failed:\n{}", stderr).red());
        Ok(false)
    }
}

fn sleep_seconds(seconds: u64) {
    // Sleep for the specified number of seconds
    sleep(Duration::from_secs(seconds));
}

fn run_binary(wait_time: u64) -> std::io::Result<()> {
    // Run the compiled binary (./output) interactively
    let mut child = Command::new("./output")
        .stdin(Stdio::inherit()) // Pass stdin from the terminal
        .stdout(Stdio::inherit()) // Pass stdout to the terminal
        .stderr(Stdio::inherit()) // Pass stderr to the terminal
        .spawn()?; // Spawn the process

    // Wait for the child process to complete and get the exit status
    let exit_status = child.wait()?;

    // Check if the program returned 5
    if let Some(9) = exit_status.code() {
        println!("{}", format!("================").bright_green().bold());

        println!(
            "{} Waiting for {} seconds...",
            "Program returned 9,".yellow().bold().italic(),
            wait_time
        );

        sleep(Duration::from_secs(wait_time));
    }

    Ok(())
}

fn clear_screen() {
    // Clears the terminal screen
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

    // Set up a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering events.
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    // Watch the file path
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    // Initial compilation
    compile_c_file(&args.file_path)?;

    println!(
        "{}",
        format!("Watching file: {}", args.file_path)
            .cyan()
            .bold()
            .underline()
    );

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let EventKind::Modify(_) = event.kind {
                    // Delay before recompiling
                    if args.delay > 0 {
                        sleep_seconds(args.delay);
                    }

                    // Clear screen if the -c flag is set
                    if args.clear_screen {
                        clear_screen();
                    }

                    println!("{}", "File changed, recompiling...".bright_blue().bold());
                    println!("{}", format!("================").bright_green().bold());

                    // Remove old executable before recompiling

                    // Recompile the C file
                    let e = compile_c_file(&args.file_path)?;

                    // Run the compiled binary and allow interactive input
                    if e {
                        println!("{}", "Running the program:".green());
                        run_binary(args.wait_time)?;
                    }
                }
            }
            Ok(Err(e)) => eprintln!("{}", format!("Watch error: {:?}", e).red()),
            Err(e) => eprintln!("{}", format!("Channel error: {:?}", e).red()),
        }
    }
}
