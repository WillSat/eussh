#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    validate_url(&url)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    Ok(())
}

fn validate_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed != url || trimmed.len() > 2048 {
        return Err("Invalid URL".to_string());
    }
    // Reject control characters and obvious injection attempts
    if trimmed.contains('\n') || trimmed.contains('\r') || trimmed.contains('\t') {
        return Err("Invalid URL: contains control characters".to_string());
    }
    // Reject username@host confusion (e.g. http://evil.com@safe.com)
    // The @ must not appear between the scheme separator and the first / after //
    let lower = trimmed.to_ascii_lowercase();
    if let Some(after_scheme) = lower.strip_prefix("http://").or_else(|| lower.strip_prefix("https://")) {
        // The authority portion is between the scheme prefix and the next /
        let authority_end = after_scheme.find('/').unwrap_or(after_scheme.len());
        let authority = &after_scheme[..authority_end];
        if authority.contains('@') {
            return Err("Invalid URL: userinfo in authority is not allowed".to_string());
        }
        if authority.is_empty() {
            return Err("Invalid URL: empty host".to_string());
        }
        Ok(())
    } else {
        Err("Only http and https URLs can be opened".to_string())
    }
}
