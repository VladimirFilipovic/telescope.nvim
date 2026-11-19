//! Tool definitions and implementations, named after telescope builtins.

use crate::fuzzy;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MAX_FILES: usize = 50;
const MAX_MATCHES: usize = 200;
const MAX_COMMITS: usize = 30;

pub fn definitions() -> Value {
    let path = json!({ "type": "string", "description": "Absolute project root to search. Defaults to the server's root." });
    json!([
        {
            "name": "find_files",
            "description": "Fuzzy-find project files by name (fzf-style ranking, respects .gitignore).",
            "inputSchema": { "type": "object", "properties": {
                "query": { "type": "string", "description": "Fuzzy query, e.g. 'bltnfiles'." },
                "limit": { "type": "integer", "description": "Max results (default 50)." },
                "path": path
            }, "required": ["query"] }
        },
        {
            "name": "live_grep",
            "description": "Search file contents with a ripgrep regex; results as path:line:col: text.",
            "inputSchema": { "type": "object", "properties": {
                "pattern": { "type": "string", "description": "Rust regex (smart case)." },
                "glob": { "type": "string", "description": "Optional file glob, e.g. '*.lua'." },
                "limit": { "type": "integer", "description": "Max matches (default 200)." },
                "path": path
            }, "required": ["pattern"] }
        },
        {
            "name": "grep_string",
            "description": "Search for an exact string (no regex), e.g. a symbol name.",
            "inputSchema": { "type": "object", "properties": {
                "text": { "type": "string" },
                "word": { "type": "boolean", "description": "Match whole words only." },
                "path": path
            }, "required": ["text"] }
        },
        {
            "name": "git_status",
            "description": "Changed files in the work tree plus a diff stat against HEAD.",
            "inputSchema": { "type": "object", "properties": { "path": path } }
        },
        {
            "name": "git_commits",
            "description": "Recent commits, optionally only those touching one file (git_bcommits).",
            "inputSchema": { "type": "object", "properties": {
                "count": { "type": "integer", "description": "Number of commits (default 30)." },
                "file": { "type": "string", "description": "Only commits touching this file." },
                "path": path
            } }
        },
        {
            "name": "todo",
            "description": "TODO/FIXME/HACK/XXX/NOTE/BUG markers across the project.",
            "inputSchema": { "type": "object", "properties": { "path": path } }
        }
    ])
}

pub fn call(name: &str, args: &Value) -> Result<String, String> {
    let root = root(args)?;
    let s = |k: &str| args.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let n = |k: &str, d: usize| args.get(k).and_then(Value::as_u64).map_or(d, |v| (v as usize).max(1));
    match name {
        "find_files" => find_files(&root, &s("query"), n("limit", MAX_FILES)),
        "live_grep" => {
            let pattern = s("pattern");
            if pattern.is_empty() {
                return Err("pattern is required".into());
            }
            let mut rg = vec!["-e".to_string(), pattern];
            let glob = s("glob");
            if !glob.is_empty() {
                rg.extend(["--glob".into(), glob]);
            }
            grep(&root, rg, n("limit", MAX_MATCHES))
        }
        "grep_string" => {
            let text = s("text");
            if text.is_empty() {
                return Err("text is required".into());
            }
            let mut rg = vec!["--fixed-strings".to_string()];
            if args.get("word").and_then(Value::as_bool).unwrap_or(false) {
                rg.push("--word-regexp".into());
            }
            rg.extend(["-e".into(), text]);
            grep(&root, rg, MAX_MATCHES)
        }
        "todo" => grep(&root, vec!["-e".into(), r"\b(TODO|FIXME|HACK|XXX|NOTE|BUG)\b".into()], MAX_MATCHES),
        "git_status" => git_status(&root),
        "git_commits" => git_commits(&root, n("count", MAX_COMMITS), &s("file")),
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Search root: the call's `path`, else TELESCOPE_ROOT, else the working directory.
fn root(args: &Value) -> Result<PathBuf, String> {
    let dir = args
        .get("path")
        .and_then(Value::as_str)
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TELESCOPE_ROOT").map(PathBuf::from))
        .map_or_else(std::env::current_dir, Ok)
        .map_err(|e| e.to_string())?;
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(format!("not a directory: {}", dir.display()))
    }
}

/// Output of a finished command run in `root`.
fn run(root: &Path, program: &str, args: &[String]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("failed to run {program}: {e} (is it on PATH?)"))?;
    match out.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        // rg exits 1 when it found nothing.
        Some(1) if program == "rg" => Ok(String::new()),
        _ => Err(format!("{program} failed: {}", String::from_utf8_lossy(&out.stderr).trim())),
    }
}

fn find_files(root: &Path, query: &str, limit: usize) -> Result<String, String> {
    let listing = run(root, "rg", &["--files".into(), "--hidden".into(), "--glob".into(), "!.git".into()])?;
    let total = listing.lines().count();
    let ranked = fuzzy::rank(query, listing.lines(), limit);
    let mut text = format!("{} of {total} files match `{query}` in {}\n", ranked.len(), root.display());
    for f in ranked {
        text.push_str(f);
        text.push('\n');
    }
    Ok(text)
}

/// Runs rg and keeps the first `limit` matches; rg is stopped there so a broad
/// pattern on a large repository stays cheap.
fn grep(root: &Path, extra: Vec<String>, limit: usize) -> Result<String, String> {
    let mut child = Command::new("rg")
        .args([
            "--line-number", "--column", "--no-heading", "--color=never", "--smart-case",
            "--hidden", "--glob", "!.git", "--max-columns=240", "--max-columns-preview", "--no-messages",
        ])
        .args(extra)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run rg: {e} (is it on PATH?)"))?;
    let stdout = child.stdout.take().ok_or("rg: no stdout")?;
    let mut lines = Vec::new();
    let mut truncated = false;
    for line in BufReader::new(stdout).split(b'\n') {
        let Ok(line) = line else { break };
        if lines.len() == limit {
            truncated = true;
            break;
        }
        lines.push(String::from_utf8_lossy(&line).into_owned());
    }
    if truncated {
        let _ = child.kill();
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    // rg exits 1 when nothing matched and 2 on a real error such as a bad regex.
    if lines.is_empty() {
        return match out.status.code() {
            Some(0 | 1) => Ok("no matches".into()),
            _ => Err(format!("rg failed: {}", String::from_utf8_lossy(&out.stderr).trim())),
        };
    }
    let header = if truncated {
        format!("first {limit} matches in {} (more exist; narrow the search)", root.display())
    } else {
        format!("{} matches in {}", lines.len(), root.display())
    };
    Ok(format!("{header}\n{}\n", lines.join("\n")))
}

fn git_status(root: &Path) -> Result<String, String> {
    let status = run(root, "git", &["status".into(), "--short".into(), "--branch".into()])?;
    let stat = run(root, "git", &["diff".into(), "HEAD".into(), "--stat".into()]).unwrap_or_default();
    Ok(format!("{status}\n{stat}"))
}

fn git_commits(root: &Path, count: usize, file: &str) -> Result<String, String> {
    let mut args: Vec<String> = ["log", "--date=short", "--format=%h %ad %s <%an>"].map(String::from).to_vec();
    args.push(format!("-n{count}"));
    if !file.is_empty() {
        args.extend(["--follow".into(), "--".into(), file.into()]);
    }
    run(root, "git", &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_defined_tool_is_callable() {
        let missing = json!({ "path": "/nonexistent-telescope-root" });
        for tool in definitions().as_array().unwrap() {
            let name = tool["name"].as_str().unwrap();
            let err = call(name, &missing).unwrap_err();
            assert!(err.starts_with("not a directory"), "{name}: {err}");
        }
    }

    #[test]
    fn unknown_tool_is_an_error() {
        let here = json!({ "path": env!("CARGO_MANIFEST_DIR") });
        assert!(call("nope", &here).unwrap_err().contains("unknown tool"));
    }

    #[test]
    fn required_arguments_are_checked() {
        let here = json!({ "path": env!("CARGO_MANIFEST_DIR") });
        assert_eq!(call("live_grep", &here).unwrap_err(), "pattern is required");
        assert_eq!(call("grep_string", &here).unwrap_err(), "text is required");
    }
}
