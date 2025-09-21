# Discord Rich Presence App

A comprehensive Discord Rich Presence application built in Rust that displays system information, active applications, custom messages, and more with advanced filtering and customization options.

## Features

- **Real-time System Stats**: CPU usage, memory usage, process count, uptime
- **Application Monitoring**: Track running applications and their resource usage
- **Custom Messages**: Set personalized status messages with word filtering
- **Time Display**: Show current local time in your Discord status
- **Advanced Filtering**: Blacklist words and processes, filter by CPU usage
- **Modern GUI**: Clean, responsive interface built with egui
- **Configuration Management**: Save/load settings with JSON configuration
- **Cross-platform**: Works on Windows, macOS, and Linux

## Setup Instructions

### 1. Discord Application Setup

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name
3. Go to "Rich Presence" → "Art Assets"
4. Upload a "default" image (this will be your main icon)
5. Copy your Application ID and replace `APP_ID` in `main.rs`

### 2. Building the Application

```bash
# Clone or create the project
cargo new discord-rich-presence-app
cd discord-rich-presence-app

# Copy the provided Cargo.toml and main.rs files
# Then build the application
cargo build --release

# Run the application
cargo run --release
```

### 3. First Run Configuration

1. Start the application
2. Click "Settings" to configure your preferences
3. Set up word filtering by adding blacklisted words
4. Configure activity filters to hide unwanted processes
5. Adjust update intervals and display options
6. Click "Save Config" to persist your settings

## Configuration Options

### Display Settings

- **Show System Stats**: Display CPU and memory usage
- **Show Time**: Show current local time
- **Show Applications**: Display currently running applications
- **Update Interval**: How often to refresh the Discord status (5-300 seconds)

### Word Filtering

- Add words to blacklist that will be replaced with `[FILTERED]`
- Useful for hiding sensitive information from process names or window titles
- Default blacklisted words: "password", "secret", "private"

### Activity Filters

- **Hide System Processes**: Filter out system processes like `dwm.exe`, `csrss.exe`
- **Minimum CPU Usage**: Only show processes using more than X% CPU
- **Blacklisted Processes**: Specific processes to always hide

### Custom Messages

- Set a custom message that overrides system-generated details
- Subject to word filtering for privacy
- Can be cleared at any time

## Usage

1. **Connect**: The app automatically connects to Discord on startup
2. **Monitor**: View real-time system stats and process information
3. **Customize**: Use custom messages or let the app auto-generate status
4. **Filter**: Words and processes are automatically filtered based on your settings
5. **Preview**: See exactly what will be displayed on Discord in the Activity Preview section

## File Structure

```
discord-rich-presence-app/
├── Cargo.toml          # Dependencies and project configuration
├── src/
│   └── main.rs         # Main application code
└── config/             # Auto-created configuration directory
    └── config.json     # Saved settings
```

## Platform-Specific Features

### Windows

- Active window title detection
- Full process monitoring with CPU/memory stats
- System tray integration (planned)

### macOS/Linux

- Basic process monitoring
- Active window detection (requires additional setup)

## Troubleshooting

### Discord Connection Issues

1. Ensure Discord is running
2. Check that your Application ID is correct
3. Verify the Discord app exists in the Developer Portal
4. Try clicking "Reconnect"

### Performance Issues

1. Increase the update interval in settings
2. Enable "Hide Background Apps" to reduce process monitoring
3. Increase minimum CPU usage threshold

### Privacy Concerns

1. Add sensitive words to the blacklist
2. Add private applications to the process blacklist
3. Use custom messages instead of auto-generated content

## Building for Production

```bash
# Build optimized release version
cargo build --release

# The executable will be in target/release/
# Copy it anywhere and run - it's self-contained
```

## Dependencies

- `discord-rich-presence`: Discord RPC client
- `egui/eframe`: Cross-platform GUI framework
- `sysinfo`: System and process monitoring
- `chrono`: Date and time handling
- `tokio`: Async runtime for background tasks
- `serde/serde_json`: Configuration serialization
- `regex`: Text filtering and pattern matching

## Contributing

Feel free to submit issues and enhancement requests! Areas for improvement:

- Additional platform-specific features
- More customization options
- Plugin system for custom data sources
- Better error handling and recovery
- Themes and UI customization

## License

This project is open source. Choose an appropriate license for your needs.
