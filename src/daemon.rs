use std::fs;
use std::path::PathBuf;
use directories::BaseDirs;

pub fn setup_daemon() -> std::io::Result<()> {
    if let Some(base_dirs) = BaseDirs::new() {
        // Get the LaunchAgents directory
        let launch_agents_dir = base_dirs.home_dir().join("Library/LaunchAgents");
        fs::create_dir_all(&launch_agents_dir)?;

        // Get the path where cargo installed our binary
        let cargo_bin_path = base_dirs.home_dir()
            .join(".cargo/bin/mac-clip")
            .to_string_lossy()
            .to_string();

        // Create the plist content
        let plist_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mac-clip.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/mac-clip.stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/mac-clip.stderr.log</string>
</dict>
</plist>"#, cargo_bin_path);

        // Write the plist file
        let plist_path = launch_agents_dir.join("com.mac-clip.daemon.plist");
        fs::write(&plist_path, plist_content)?;

        // Load the launch agent
        std::process::Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&plist_path)
            .output()?;

        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not determine user directories",
        ))
    }
}
