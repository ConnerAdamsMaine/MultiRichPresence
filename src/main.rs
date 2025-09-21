use chrono::{DateTime, Local};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use eframe::egui;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::sync::mpsc;

const APP_ID: &str = "1234567890123456789"; // Replace with your Discord app ID

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub blacklisted_words: Vec<String>,
    pub show_system_stats: bool,
    pub show_time: bool,
    pub show_applications: bool,
    pub custom_messages: Vec<String>,
    pub update_interval_seconds: u64,
    pub discord_app_id: String,
    pub activity_filters: ActivityFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFilters {
    pub hide_system_processes: bool,
    pub hide_background_apps: bool,
    pub minimum_cpu_usage: f32,
    pub blacklisted_processes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blacklisted_words: vec![
                "password".to_string(),
                "secret".to_string(),
                "private".to_string(),
            ],
            show_system_stats: true,
            show_time: true,
            show_applications: true,
            custom_messages: vec!["Working on something cool".to_string()],
            update_interval_seconds: 15,
            discord_app_id: APP_ID.to_string(),
            activity_filters: ActivityFilters::default(),
        }
    }
}

impl Default for ActivityFilters {
    fn default() -> Self {
        Self {
            hide_system_processes: true,
            hide_background_apps: true,
            minimum_cpu_usage: 0.1,
            blacklisted_processes: vec![
                "dwm.exe".to_string(),
                "winlogon.exe".to_string(),
                "csrss.exe".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub uptime: u64,
    pub process_count: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub start_time: u64,
}

#[derive(Debug, Clone)]
pub struct ActivityData {
    pub system_stats: SystemStats,
    pub current_time: DateTime<Local>,
    pub top_processes: Vec<ProcessInfo>,
    pub active_window: Option<String>,
}

pub struct DiscordRpcApp {
    config: Config,
    activity_data: Arc<Mutex<Option<ActivityData>>>,
    discord_client: Option<DiscordIpcClient>,
    system: System,
    last_update: Instant,
    word_filter: Regex,
    
    // UI state
    custom_message_input: String,
    show_config: bool,
    new_blacklisted_word: String,
    new_custom_message: String,
    connection_status: String,
    
    // Channels for communication
    activity_sender: Option<mpsc::UnboundedSender<ActivityData>>,
}

impl DiscordRpcApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Self::load_config().unwrap_or_default();
        let word_filter = Self::create_word_filter(&config.blacklisted_words);
        
        let mut app = Self {
            config,
            activity_data: Arc::new(Mutex::new(None)),
            discord_client: None,
            system: System::new_all(),
            last_update: Instant::now(),
            word_filter,
            custom_message_input: String::new(),
            show_config: false,
            new_blacklisted_word: String::new(),
            new_custom_message: String::new(),
            connection_status: "Disconnected".to_string(),
            activity_sender: None,
        };
        
        app.connect_discord();
        app.start_system_monitoring();
        app
    }
    
    fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let dirs = directories::ProjectDirs::from("com", "multirichpresence", "MultiRichPresence")
            .ok_or("Could not find project directory")?;
        
        let config_path = dirs.config_dir().join("config.json");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }
    
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dirs = directories::ProjectDirs::from("com", "multirichpresence", "MultiRichPresence")
            .ok_or("Could not find project directory")?;
        
        std::fs::create_dir_all(dirs.config_dir())?;
        let config_path = dirs.config_dir().join("config.json");
        
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    fn create_word_filter(blacklisted_words: &[String]) -> Regex {
        if blacklisted_words.is_empty() {
            return Regex::new(r"(?i)^$").unwrap(); // Match nothing
        }
        
        let pattern = format!(
            r"(?i)\b({})\b",
            blacklisted_words.join("|")
        );
        
        Regex::new(&pattern).unwrap_or_else(|_| Regex::new(r"(?i)^$").unwrap())
    }
    
    fn connect_discord(&mut self) {
        match DiscordIpcClient::new(&self.config.discord_app_id) {
            Ok(mut client) => {
                match client.connect() {
                    Ok(_) => {
                        self.discord_client = Some(client);
                        self.connection_status = "Connected".to_string();
                        log::info!("Connected to Discord RPC");
                    }
                    Err(e) => {
                        self.connection_status = format!("Connection failed: {}", e);
                        log::error!("Failed to connect to Discord: {}", e);
                    }
                }
            }
            Err(e) => {
                self.connection_status = format!("Client creation failed: {}", e);
                log::error!("Failed to create Discord client: {}", e);
            }
        }
    }
    
    fn start_system_monitoring(&mut self) {
        let activity_data = Arc::clone(&self.activity_data);
        let config = self.config.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.activity_sender = Some(tx);
        
        tokio::spawn(async move {
            let mut system = System::new_all();
            let mut interval = tokio::time::interval(Duration::from_secs(config.update_interval_seconds));
            
            loop {
                interval.tick().await;
                
                system.refresh_all();
                
                let stats = SystemStats {
                    cpu_usage: system.global_cpu_info().cpu_usage(),
                    memory_usage: (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
                    memory_total: system.total_memory(),
                    memory_used: system.used_memory(),
                    uptime: System::uptime(),
                    process_count: system.processes().len(),
                };
                
                let mut processes: Vec<ProcessInfo> = Vec::new();
                
                for (pid, process) in system.processes() {
                    if config.activity_filters.hide_system_processes {
                        if config.activity_filters.blacklisted_processes.contains(&process.name().to_string()) {
                            continue;
                        }
                    }
                    
                    if process.cpu_usage() >= config.activity_filters.minimum_cpu_usage {
                        processes.push(ProcessInfo {
                            name: process.name().to_string(),
                            pid: pid.as_u32(),
                            cpu_usage: process.cpu_usage(),
                            memory_usage: process.memory(),
                            start_time: process.start_time(),
                        });
                    }
                }
                
                processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
                processes.truncate(5); // Keep top 5 processes
                
                let activity = ActivityData {
                    system_stats: stats,
                    current_time: Local::now(),
                    top_processes: processes,
                    active_window: Self::get_active_window_title(),
                };
                
                if let Ok(mut data) = activity_data.lock() {
                    *data = Some(activity);
                }
            }
        });
    }
    
    #[cfg(windows)]
    fn get_active_window_title() -> Option<String> {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};
        
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            
            let mut buffer: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
            
            if len > 0 {
                let os_string = OsString::from_wide(&buffer[..len as usize]);
                os_string.into_string().ok()
            } else {
                None
            }
        }
    }
    
    #[cfg(not(windows))]
    fn get_active_window_title() -> Option<String> {
        None // Implement for other platforms as needed
    }
    
    fn filter_text(&self, text: &str) -> String {
        self.word_filter.replace_all(text, "[FILTERED]").to_string()
    }
    
    fn update_discord_activity(&mut self) {
    // Do all the filtering BEFORE borrowing discord_client mutably
    let activity_data = self.activity_data.lock().unwrap();
    
    if let Some(ref data) = *activity_data {
        let mut details = String::new();
        let mut state = String::new();
        
        // Pre-filter all text before mutable borrow
        let filtered_custom_message = if !self.custom_message_input.is_empty() {
            Some(self.filter_text(&self.custom_message_input))
        } else {
            None
        };
        
        let filtered_process_name = if self.config.show_applications && !data.top_processes.is_empty() {
            Some(self.filter_text(&data.top_processes[0].name))
        } else {
            None
        };
        
        // Build details and state strings
        if self.config.show_system_stats {
            details = format!(
                "CPU: {:.1}% | RAM: {:.1}%",
                data.system_stats.cpu_usage,
                data.system_stats.memory_usage
            );
        }
        
        if self.config.show_time {
            if !state.is_empty() {
                state.push_str(" | ");
            }
            state.push_str(&format!("Time: {}", data.current_time.format("%H:%M:%S")));
        }
        
        if let Some(ref process_name) = filtered_process_name {
            if !state.is_empty() {
                state.push_str(" | ");
            }
            state.push_str(&format!("Running: {}", process_name));
        }
        
        // NOW do the mutable borrow
        if let Some(ref mut client) = self.discord_client {
            let mut activity_builder = activity::Activity::new();
            
            if let Some(ref message) = filtered_custom_message {
                activity_builder = activity_builder.details(message);
            } else if !details.is_empty() {
                activity_builder = activity_builder.details(&details);
            }
            
            if !state.is_empty() {
                activity_builder = activity_builder.state(&state);
            }
            
            activity_builder = activity_builder.timestamps(
                activity::Timestamps::new().start(data.current_time.timestamp())
            );
            
            activity_builder = activity_builder.assets(
                activity::Assets::new()
                    .large_image("default")
                    .large_text("MultiRichPresence")
            );
            
            if let Err(e) = client.set_activity(activity_builder) {
                log::error!("Failed to set Discord activity: {}", e);
                self.connection_status = format!("Activity update failed: {}", e);
            }
        }
    }
}

impl eframe::App for DiscordRpcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update Discord activity periodically
        if self.last_update.elapsed() >= Duration::from_secs(self.config.update_interval_seconds) {
            self.update_discord_activity();
            self.last_update = Instant::now();
        }
        
        // Request repaint for real-time updates
        ctx.request_repaint_after(Duration::from_secs(1));
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MultiRichPresence");
            
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(
                    if self.connection_status == "Connected" {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    },
                    &self.connection_status,
                );
                
                if ui.button("Reconnect").clicked() {
                    self.connect_discord();
                }
                
                ui.separator();
                
                if ui.button("Settings").clicked() {
                    self.show_config = !self.show_config;
                }
            });
            
            ui.separator();
            
            // Custom message input
            ui.horizontal(|ui| {
                ui.label("Custom Message:");
                ui.text_edit_singleline(&mut self.custom_message_input);
                if ui.button("Clear").clicked() {
                    self.custom_message_input.clear();
                }
            });
            
            ui.separator();
            
            // Activity preview
            ui.collapsing("Activity Preview", |ui| {
                if let Ok(activity_data) = self.activity_data.lock() {
                    if let Some(ref data) = *activity_data {
                        ui.label(format!("CPU Usage: {:.1}%", data.system_stats.cpu_usage));
                        ui.label(format!("Memory Usage: {:.1}% ({} MB / {} MB)", 
                            data.system_stats.memory_usage,
                            data.system_stats.memory_used / 1024 / 1024,
                            data.system_stats.memory_total / 1024 / 1024
                        ));
                        ui.label(format!("Process Count: {}", data.system_stats.process_count));
                        ui.label(format!("Current Time: {}", data.current_time.format("%Y-%m-%d %H:%M:%S")));
                        
                        if !data.top_processes.is_empty() {
                            ui.label("Top Processes:");
                            for process in &data.top_processes {
                                ui.label(format!("  {} - {:.1}% CPU", 
                                    self.filter_text(&process.name), 
                                    process.cpu_usage
                                ));
                            }
                        }
                        
                        if let Some(ref window) = data.active_window {
                            ui.label(format!("Active Window: {}", self.filter_text(window)));
                        }
                    } else {
                        ui.label("No activity data available");
                    }
                }
            });
            
            // Configuration panel
            if self.show_config {
                ui.separator();
                ui.heading("Configuration");
                
                ui.checkbox(&mut self.config.show_system_stats, "Show System Stats");
                ui.checkbox(&mut self.config.show_time, "Show Time");
                ui.checkbox(&mut self.config.show_applications, "Show Applications");
                
                ui.horizontal(|ui| {
                    ui.label("Update Interval (seconds):");
                    ui.add(egui::Slider::new(&mut self.config.update_interval_seconds, 5..=300));
                });
                
                ui.collapsing("Word Filter", |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_blacklisted_word);
                        if ui.button("Add Blacklisted Word").clicked() && !self.new_blacklisted_word.is_empty() {
                            self.config.blacklisted_words.push(self.new_blacklisted_word.clone());
                            self.new_blacklisted_word.clear();
                            self.word_filter = Self::create_word_filter(&self.config.blacklisted_words);
                        }
                    });
                    
                    let mut to_remove = None;
                    for (i, word) in self.config.blacklisted_words.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(word);
                            if ui.button("Remove").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }
                    
                    if let Some(index) = to_remove {
                        self.config.blacklisted_words.remove(index);
                        self.word_filter = Self::create_word_filter(&self.config.blacklisted_words);
                    }
                });
                
                ui.collapsing("Activity Filters", |ui| {
                    ui.checkbox(&mut self.config.activity_filters.hide_system_processes, "Hide System Processes");
                    ui.checkbox(&mut self.config.activity_filters.hide_background_apps, "Hide Background Apps");
                    
                    ui.horizontal(|ui| {
                        ui.label("Minimum CPU Usage:");
                        ui.add(egui::Slider::new(&mut self.config.activity_filters.minimum_cpu_usage, 0.0..=10.0).suffix("%"));
                    });
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Save Config").clicked() {
                        if let Err(e) = self.save_config() {
                            log::error!("Failed to save config: {}", e);
                        }
                    }
                    
                    if ui.button("Reset to Default").clicked() {
                        self.config = Config::default();
                        self.word_filter = Self::create_word_filter(&self.config.blacklisted_words);
                    }
                });
            }
        });
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(ref mut client) = self.discord_client {
            let _ = client.clear_activity();
            let _ = client.close();
        }
        
        let _ = self.save_config();
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };
    
    eframe::run_native(
        "MultiRichPresence",
        options,
        Box::new(|cc| Box::new(DiscordRpcApp::new(cc))),
    )
}