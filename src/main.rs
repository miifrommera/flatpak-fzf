use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::env;

fn main() {
    // Get any command-line arguments passed to the program
    let args: Vec<String> = env::args().skip(1).collect();
    let additional_args = args.join(" ");

    match flatpak_list() {
        Ok(app_list) => {
            if app_list.is_empty() {
                println!("No Flatpak applications found.");
                return;
            }
            match fzf_search(&app_list) {
                Ok(selected_app) => {
                    if !selected_app.is_empty() {
                        let selected_id = extract_app_id(&selected_app);
                        
                        // Build the command
                        let flatpak_cmd = if additional_args.is_empty() {
                            format!("flatpak run {}", selected_id)
                        } else {
                            format!("flatpak run {} {}", selected_id, additional_args)
                        };
                        
                        // Show the command that will be run
                        println!("Running: {}", flatpak_cmd);
                        
                        // Execute flatpak run directly with all stdio inherited
                        // This ensures terminal control is properly passed to the application
                        let mut cmd_parts = flatpak_cmd.split_whitespace();
                        let cmd_name = cmd_parts.next().unwrap_or("flatpak");
                        
                        let status = Command::new(cmd_name)
                            .args(cmd_parts)
                            .stdin(Stdio::inherit())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .status();
                            
                        match status {
                            Ok(exit_status) => {
                                if !exit_status.success() {
                                    eprintln!("Command failed with exit code: {:?}", exit_status.code());
                                }
                            },
                            Err(e) => eprintln!("Failed to execute command: {}", e),
                        }
                    } else {
                        println!("No application selected.");
                    }
                }
                Err(e) => eprintln!("Error using fzf: {}", e),
            }
        }
        Err(e) => eprintln!("Error listing Flatpak apps: {}", e),
    }
}

fn flatpak_list() -> Result<Vec<String>, io::Error> {
    // Use a different command to get more detailed info about apps
    // Using --columns to specify exactly what we want
    let output = Command::new("flatpak")
        .args(["list", "--app", "--columns=name,application,version,branch,origin"])
        .output()?;
        
    if output.status.success() {
        let app_list = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| format_columns(line))
            .collect();
        Ok(app_list)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to list Flatpak apps",
        ))
    }
}

fn format_columns(line: &str) -> String {
    // Split by tabs since the flatpak command with --columns uses tab as delimiter
    let parts: Vec<&str> = line.split('\t').collect();
    
    if parts.len() >= 5 {
        // Properly named columns from the --columns flag
        let mut name = parts[0].trim().to_string();
        if name.len() > 20 {
            name = format!("{}..", &name[..17]);
        }
        let name = format!("{:<20}", name);
        
        let mut app_id = parts[1].trim().to_string();
        if app_id.len() > 34 {
            app_id = format!("{}..", &app_id[..31]);
        }
        let app_id = format!("{:<40}", app_id);
        
        // This is now the actual app version, not the remote version
        let mut version = parts[2].trim().to_string();
        if version.len() > 10 {
            version = format!("{}..", &version[..7]);
        }
        let version = format!("{:<10}", version);
        
        let branch = format!("{:<10}", parts[3].trim());
        let origin = format!("{:<20}", parts[4].trim());
        
        format!(
            "{} {} {} {} {}",
            name, app_id, version, branch, origin
        )
    } else {
        // Fallback in case the format is unexpected
        line.to_string()
    }
}

fn fzf_search(app_list: &[String]) -> Result<String, io::Error> {
    let mut fzf = Command::new("fzf")
        .arg("--height=20%")
        .arg("--reverse")
        .arg("--inline-info")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let fzf_stdin = fzf.stdin.as_mut().unwrap();
        write!(fzf_stdin, "Name                 AppID                                    Version     Branch     Origin               \n")?;
        write!(fzf_stdin, "{}", app_list.join("\n"))?;
    }
    let output = fzf.wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fzf failed"))
    }
}

fn extract_app_id(selected_app: &str) -> String {
    let parts: Vec<&str> = selected_app.split_whitespace().collect();
    // AppID is still in the second position (index 1)
    if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "".to_string()
    }
}