use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
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
                        print!("flatpak run {}", selected_id);
                        io::stdout().flush().unwrap(); // Ensure the prompt stays on the same line

                        // Wait for input or Enter to execute
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();

                        let full_command = format!("flatpak run {} {}", selected_id, input.trim());
                        let mut child = Command::new("sh")
                            .arg("-c")
                            .arg(&full_command)
                            .spawn()
                            .expect("Failed to execute command");

                        child.wait().expect("Command wasn't running");
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
    let output = Command::new("flatpak").arg("list").arg("--app").output()?;
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
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 6 {
        let mut name = parts[..parts.len() - 5].join(" ");
        if name.len() > 20 {
            name = format!("{}...", &name[..17]);
        }
        let name = format!("{:<20}", name);

        let mut app_id = parts[parts.len() - 5].to_string();
        if app_id.len() > 34 {
            app_id = format!("{}...", &app_id[..31]);
        }
        let app_id = format!("{:<40}", app_id);

        let mut version = parts[parts.len() - 4].to_string();
        if version.len() > 10 {
            version = format!("{}...", &version[..7]);
        }
        let version = format!("{:<10}", version);

        let branch = format!("{:<10}", parts[parts.len() - 3]);
        let origin = format!("{:<20}", parts[parts.len() - 2]);
        let installation = format!("{:<15}", parts[parts.len() - 1]);
        format!(
            "{} {} {} {} {} {}",
            name, app_id, version, branch, origin, installation
        )
    } else {
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
        write!(fzf_stdin, "Name                 AppID                                    Version     Branch     Origin               Installation    \n")?;
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
    if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "".to_string()
    }
}
