use crossterm::style::{SetForegroundColor, SetBackgroundColor, Color};
use crossterm::ExecutableCommand;
use std::process::{Command, Stdio};
use std::io::{stdout, Write, stdin};
use std::collections::HashMap;
use std::path::Path;
use ctrlc; // For handling Ctrl+C gracefully
use env_logger; // For logging

fn set_text_color() {
    stdout()
        .execute(SetForegroundColor(Color::Rgb { r: 255, g: 165, b: 0 }))
        .unwrap(); // Set text to orange
}

fn reset_colors() {
    stdout().execute(SetForegroundColor(Color::Reset)).unwrap();
    stdout().execute(SetBackgroundColor(Color::Reset)).unwrap();
}

fn display_logo() {
    let logo = r#"
 ###########################              
#TTTTT III TTTTT AAAAA N   N#                
#  T    I    T   AAAAA N N N#                 
#  T    I    T   A   A N  NN#              
#  T   III   T   A   A N   N#
############################
          ########
           ########
            ########
             ########
              ########
               ########
                ########
                 ########
    "#;

    stdout().execute(SetBackgroundColor(Color::Black)).unwrap(); // Set background to black
    set_text_color();
    println!("{}", logo);
    reset_colors();
    display_prompt();
}

fn display_prompt() {
    set_text_color();
    print!("Titan> ");
    stdout().flush().unwrap();
}

fn execute_kernel_command(command: &str, args: Vec<&str>) -> Result<(), String> {
    let output = Command::new(command)
        .args(args)
        .stdout(Stdio::inherit()) // Pass stdout directly to terminal
        .stderr(Stdio::inherit()) // Pass stderr directly to terminal
        .status();

    match output {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(format!("Command exited with non-zero status")),
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

fn build_kernel() {
    if let Err(e) = execute_kernel_command("make", vec!["-j4"]) {
        eprintln!("{}", e);
    }
}

fn handle_command(input: &str, commands: &HashMap<&str, Box<dyn Fn()>>) {
    let trimmed_input = input.trim();

    // Check if the input matches a predefined command
    if let Some(func) = commands.get(trimmed_input) {
        func();
        return; // Exit after executing the predefined command
    }

    // Execute the input as a shell command
    let status = Command::new("/bin/bash")
        .arg("-c")
        .arg(trimmed_input)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    // Handle the result of the shell command execution
    match status {
        Ok(status) if status.success() => (), // Shell command executed successfully
        Ok(_) => eprintln!("Shell command exited with a non-zero status"),
        Err(e) => eprintln!("Failed to execute shell command: {}", e),
    }
}


fn get_ip_address() -> Option<String> {
    let output = Command::new("ip")
        .arg("addr")
        .output()
        .expect("Failed to execute 'ip addr'");

    if !output.status.success() {
        eprintln!(
            "Error executing 'ip addr': {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("inet") && !line.contains("127.0.0.1") && !line.contains("::1") {
            if let Some(ip_section) = line.split_whitespace().nth(1) {
                if let Some(ip) = ip_section.split('/').next() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

fn run_ip_with_feedback() {
    if let Err(e) = run_ip() {
        eprintln!("Error running IP script: {}", e);
    }

    match get_ip_address() {
        Some(ip) => println!("Updated IP Address: {}", ip),
        None => println!("Failed to fetch updated IP address"),
    }
}

fn run_ip() -> Result<(), String> {
    let script_path = "./src/ipBounce.sh";
    if !Path::new(script_path).exists() {
        return Err(format!("Script not found at path: {}", script_path));
    }

    let output = Command::new("bash")
        .arg(script_path)
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Script execution failed with status: {:?}. Error: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn setup_logging() {
    env_logger::init();
    log::info!("Logging initialized successfully.");
}

fn setup_ctrlc_handler() {
    ctrlc::set_handler(move || {
        println!("\nTitan terminal is shutting down. Goodbye!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn main() {
    setup_logging();
    setup_ctrlc_handler();

    display_logo();
    run_ip_with_feedback();

    let mut commands: HashMap<&str, Box<dyn Fn()>> = HashMap::new();
    commands.insert("build", Box::new(build_kernel));
    commands.insert("exit", Box::new(|| std::process::exit(0)));
    commands.insert("Titan-ip", Box::new(|| {
        match get_ip_address() {
            Some(ip) => println!("Current IP Address: {}", ip),
            None => println!("Could not retrieve IP address"),
        }
    }));
    commands.insert("help", Box::new(|| {
        println!("Available commands:");
        println!("  build       - Build the kernel");
        println!("  exit        - Exit the Titan terminal");
        println!("  Titan-ip    - Display current IP address");
        println!("  help        - Show this help menu");
    }));

    loop {
        display_prompt();

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        handle_command(&input, &commands);
        reset_colors();
    }
}
