use std::process::Command;

use anyhow::Context;

/// Launch the UNAVI client.
/// This function will spawn the client process and return immediately.
pub fn launch_client() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!("failed to get executable directory"))?
        .join(if cfg!(windows) {
            "unavi-client.exe"
        } else {
            "unavi-client"
        });

    if !exe_path.exists() {
        anyhow::bail!("client executable not found at {}", exe_path.display());
    }

    Command::new(exe_path)
        .spawn()
        .context("failed to launch client")?;

    Ok(())
}

// TODO: Implement client update checking
#[allow(dead_code)]
pub fn check_client_updates() -> anyhow::Result<bool> {
    // Placeholder for future client update functionality
    Ok(false)
}
