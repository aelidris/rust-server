use std::process::{Command, Stdio};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;

pub fn execute_cgi(
    script_path: &Path,
    method: &str,
    query_string: Option<&str>,
    body: Option<&[u8]>,
    headers: &HashMap<String, String>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Determine the interpreter based on file extension
    let extension = script_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    let mut cmd = match extension {
        "py" => {
            let mut c = Command::new("python3");
            c.arg(script_path);
            c
        }
        "sh" => {
            let mut c = Command::new("sh");
            c.arg(script_path);
            c
        }
        _ => {
            // Default to direct execution if permitted
            Command::new(script_path)
        }
    };

    // Set standard CGI environment variables
    cmd.env("REQUEST_METHOD", method);
    if let Some(qs) = query_string {
        cmd.env("QUERY_STRING", qs);
    }
    if let Some(b) = body {
        cmd.env("CONTENT_LENGTH", b.len().to_string());
    }
    if let Some(content_type) = headers.get("content-type") {
        cmd.env("CONTENT_TYPE", content_type);
    }
    cmd.env("SCRIPT_FILENAME", script_path.to_string_lossy().to_string());

    // Configure input/output piping
    cmd.stdout(Stdio::piped());
    if body.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    // Spawn the child process
    let mut child = cmd.spawn()?;

    // If there is a request body (e.g., POST), write it to stdin
    if let Some(b) = body {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b)?;
        }
    }

    // Wait for the process to finish and capture output
    let output = child.wait_with_output()?;

    Ok(output.stdout)
}