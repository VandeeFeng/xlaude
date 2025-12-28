use std::fs;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};

fn which_exists(executable: &str) -> bool {
    Command::new(executable)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn resolve_editor(state: &crate::state::XlaudeState) -> Result<String> {
    if let Ok(editor) = std::env::var("EDITOR") {
        return Ok(editor);
    }

    if let Some(editor) = &state.editor {
        return Ok(editor.clone());
    }

    for editor in ["nvim", "vim", "vi", "nano"] {
        if which_exists(editor) {
            return Ok(editor.to_string());
        }
    }

    bail!("No suitable editor found. Please set EDITOR environment variable or add editor to state.json");
}

pub fn handle_config() -> Result<()> {
    let state = crate::state::XlaudeState::load()?;
    let editor = resolve_editor(&state)?;

    let parts = shell_words::split(&editor)
        .map_err(|e| anyhow!("Failed to parse EDITOR command: {editor} ({e})"))?;

    if parts.is_empty() {
        bail!("EDITOR command is empty");
    }

    let state_path = crate::state::get_state_path()?;
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let mut cmd = Command::new(&parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    cmd.arg(&state_path);

    let status = cmd
        .status()
        .with_context(|| format!("Failed to launch editor: {}", parts[0]))?;

    if !status.success() {
        bail!(
            "Editor exited with status: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "terminated by signal".to_string())
        );
    }

    Ok(())
}
