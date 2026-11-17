//! Rust model of `com.winemu.core.utils.Program` and the pure command-line
//! tokenizer in `com.winemu.core.utils.ProcessHelper.l()` (GameHub Lite
//! v5.1.8).
//!
//! Two shapes are kept:
//!
//! * [`Program`] is the executable descriptor a launcher actually needs:
//!   `path`, `args`, `env`, `cwd`. It is built from a command line with
//!   [`Program::from_command`], which uses [`split_command_line`] (the exact
//!   algorithm of `ProcessHelper.l()` — quote handling and `\ ` escapes).
//! * [`ProgramEntry`] mirrors the Java `Program` class verbatim (`label`,
//!   `icon`, `command`) with its `a()/b()/c()` accessors, `equals`, `hashCode`
//!   and `toString` equivalents.
//!
//! Actual process spawn (`ProcessHelper.g()`, `Runtime.exec`), PID discovery
//! via reflection and signal sending (`k()`/`m()`) are out of scope.

use super::EnvVars;

/// A launcher-ready executable descriptor: path, args, environment and cwd.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Executable path (`ProcessHelper.f/g`'s `cmd`/`cmdarray[0]`).
    pub path: String,
    /// Arguments following the executable.
    pub args: Vec<String>,
    /// Environment variables to pass (`ProcessHelper`'s `envp` array).
    pub env: EnvVars,
    /// Working directory (`ProcessHelper`'s `dir`/`File` argument).
    pub cwd: Option<String>,
}

impl Program {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            args: Vec::new(),
            env: EnvVars::new(),
            cwd: None,
        }
    }

    /// Build from a command line, tokenizing with [`split_command_line`].
    /// The first token becomes `path`, the rest become `args`.
    pub fn from_command(command: &str, cwd: Option<String>) -> Self {
        let mut tokens = split_command_line(command).into_iter();
        let path = tokens.next().unwrap_or_default();
        Self {
            path,
            args: tokens.collect(),
            env: EnvVars::new(),
            cwd,
        }
    }

    /// Append a single argument.
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Set an environment variable.
    pub fn env(&mut self, key: impl Into<String>, value: impl ToString) -> &mut Self {
        self.env.set(key, value);
        self
    }

    /// Set the working directory.
    pub fn cwd(&mut self, cwd: impl Into<String>) -> &mut Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// The complete argv vector: `path` followed by all `args`.
    pub fn argv(&self) -> Vec<String> {
        let mut argv = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.path.clone());
        argv.extend_from_slice(&self.args);
        argv
    }

    /// Environment as ordered `KEY=VALUE` strings (ready for a launcher).
    pub fn envp(&self) -> Vec<String> {
        self.env.export()
    }
}

/// Java `Program` data class: label, icon, command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramEntry {
    /// `f20912a` — label (`c()`).
    pub label: String,
    /// `f20913b` — icon (`b()`).
    pub icon: String,
    /// `f20914c` — command (`a()`).
    pub command: String,
}

impl ProgramEntry {
    pub fn new(label: impl Into<String>, icon: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
            command: command.into(),
        }
    }

    /// `a()` — command.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// `b()` — icon.
    pub fn icon(&self) -> &str {
        &self.icon
    }

    /// `c()` — label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Turn this menu entry into a launcher [`Program`] with an optional cwd.
    pub fn into_program(&self, cwd: Option<String>) -> Program {
        Program::from_command(&self.command, cwd)
    }

    /// Java `hashCode()`: `((label*31)+icon)*31+command`.
    pub fn java_hash_code(&self) -> i32 {
        let mut h: i32 = 0;
        h = h.wrapping_mul(31).wrapping_add(java_string_hash(&self.label));
        h = h.wrapping_mul(31).wrapping_add(java_string_hash(&self.icon));
        h = h.wrapping_mul(31).wrapping_add(java_string_hash(&self.command));
        h
    }

    /// Java `toString()`.
    pub fn java_to_string(&self) -> String {
        format!(
            "Program(label={}, icon={}, command={})",
            self.label, self.icon, self.command
        )
    }
}

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as i32);
    }
    h
}

/// `ProcessHelper.l(String)` — split a command line into arguments.
///
/// Rules (transcribed from the decompiled Java):
/// * whitespace separates tokens; consecutive spaces produce one boundary;
/// * a `"` toggles quote mode; inside quotes spaces are literal and the
///   closing quote ends the current token;
/// * `\ ` (backslash followed by a space) is an escaped literal space;
/// * a trailing separator never produces an empty trailing token.
pub fn split_command_line(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if in_quotes {
            if c == '"' {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                in_quotes = false;
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else {
            let next = if i < len - 1 { chars[i + 1] } else { '\0' };
            if c != ' ' && (c != '\\' || next != ' ') {
                cur.push(c);
                if i == len - 1 {
                    out.push(std::mem::take(&mut cur));
                }
            } else if c == '\\' {
                // c == '\\' && next == ' '
                cur.push(' ');
                i += 1;
            } else if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_simple_command() {
        assert_eq!(split_command_line("wine game.exe -v"), vec!["wine", "game.exe", "-v"]);
        assert_eq!(split_command_line("  a   b  "), vec!["a", "b"]);
        assert_eq!(split_command_line(""), Vec::<String>::new());
    }

    #[test]
    fn split_quoted_and_escaped() {
        assert_eq!(
            split_command_line("prog \"my game.exe\" --name=\"foo bar\""),
            vec!["prog", "my game.exe", "--name=foo bar"]
        );
        assert_eq!(split_command_line("prog a\\ b"), vec!["prog", "a b"]);
        assert_eq!(split_command_line("\"only one\""), vec!["only one"]);
    }

    #[test]
    fn program_descriptor() {
        let mut p = Program::from_command("wine /games/doom.exe -windowed", Some("/w".into()));
        assert_eq!(p.path, "wine");
        assert_eq!(p.args, vec!["/games/doom.exe", "-windowed"]);
        assert_eq!(p.argv(), vec!["wine", "/games/doom.exe", "-windowed"]);
        p.env("BOX64_LOG", "info").arg("-fullscreen");
        assert_eq!(p.envp(), vec!["BOX64_LOG=info"]);
        assert_eq!(p.args.len(), 3);
        assert_eq!(p.cwd.as_deref(), Some("/w"));
    }

    #[test]
    fn program_entry_matches_java() {
        let e = ProgramEntry::new("Doom", "/icon.png", "wine doom.exe");
        assert_eq!(e.label(), "Doom");
        assert_eq!(e.icon(), "/icon.png");
        assert_eq!(e.command(), "wine doom.exe");
        assert_eq!(
            e.java_to_string(),
            "Program(label=Doom, icon=/icon.png, command=wine doom.exe)"
        );
        assert_eq!(e.java_hash_code(), e.clone().java_hash_code());
        let p = e.into_program(None);
        assert_eq!(p.path, "wine");
        assert_eq!(p.args, vec!["doom.exe"]);
    }
}
