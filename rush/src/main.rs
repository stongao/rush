use std::io::{self, Write};
use std::ffi::{CStr, CString};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, ForkResult};

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

        // Fork and exec the command, inheriting stdio by default
        // Build C-compatible argv: [cmd, args...]
        let c_cmd = match CString::new(cmd) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("invalid command name (contains NUL byte)");
                continue;
            }
        };
        let c_argv_storage: Vec<CString> = std::iter::once(cmd)
            .chain(args.iter().copied())
            .map(|s| CString::new(s).unwrap_or_else(|_| CString::new("?").unwrap()))
            .collect();
        let argv: Vec<&CStr> = c_argv_storage.iter().map(|s| s.as_c_str()).collect();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(_, code)) => {
                        if code != 0 {
                            eprintln!("command exited with code {}", code);
                        }
                    }
                    Ok(WaitStatus::Signaled(_, _sig, _core)) => {
                        eprintln!("command terminated by signal");
                    }
                    Ok(_) => {
                        // Other wait statuses (stopped/continued) are not expected here
                    }
                    Err(err) => {
                        eprintln!("waitpid error: {}", err);
                    }
                }
            }
            Ok(ForkResult::Child) => {
                // Child: replace image with the command
                if let Err(err) = execvp(&c_cmd, &argv) {
                    eprintln!("failed to execute '{}': {}", cmd, err);
                    unsafe { libc::_exit(127) };
                }
                unreachable!();
            }
            Err(err) => {
                eprintln!("fork error: {}", err);
            }
        }
    }
}
