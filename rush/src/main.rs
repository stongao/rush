use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt and flush
        if write!(stdout, "rush> ").is_ok() {
            let _ = stdout.flush();
        }

        // Read a single line of input
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl-D). Exit gracefully
                writeln!(stdout).ok();
                break;
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("read error: {}", err);
                continue;
            }
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        // Simple whitespace tokenization
        let mut parts = line.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => continue,
        };

        // Built-in: exit
        if cmd == "exit" {
            // Optional argument: exit code
            let code = parts
                .next()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            std::process::exit(code);
        }

        let args: Vec<&str> = parts.collect();

        // Spawn the command in blocking fashion, inheriting stdio so that
        // child stdout/stderr are displayed directly
        let status = Command::new(cmd)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(s) if s.success() => {
                // Successful execution; nothing to report
            }
            Ok(s) => {
                match s.code() {
                    Some(code) => eprintln!("command exited with code {}", code),
                    None => eprintln!("command terminated by signal"),
                }
            }
            Err(err) => {
                eprintln!("failed to execute '{}': {}", cmd, err);
            }
        }
    }
}
