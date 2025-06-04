//! Development server with hot reload and debugging features
//! 
//! This module provides a comprehensive development server for Lumos.ai projects

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::mpsc;
use crate::Result;
use super::{ProjectConfig, CliUtils};

/// Development server configuration
#[derive(Debug, Clone)]
pub struct DevServerConfig {
    pub port: u16,
    pub hot_reload: bool,
    pub debug: bool,
    pub project_root: PathBuf,
    pub watch_paths: Vec<PathBuf>,
    pub ignore_patterns: Vec<String>,
}

/// Development server state
pub struct DevServer {
    config: DevServerConfig,
    project_config: Arc<RwLock<ProjectConfig>>,
    is_running: Arc<RwLock<bool>>,
}

impl DevServer {
    /// Create a new development server
    pub fn new(config: DevServerConfig, project_config: ProjectConfig) -> Self {
        Self {
            config,
            project_config: Arc::new(RwLock::new(project_config)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the development server
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        CliUtils::success(&format!("Development server started on port {}", self.config.port));
        
        // Start file watcher if hot reload is enabled
        if self.config.hot_reload {
            self.start_file_watcher().await?;
        }

        // Start the main server loop
        self.run_server_loop().await?;

        Ok(())
    }

    /// Start file watcher for hot reload
    async fn start_file_watcher(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| crate::Error::Other(format!("Failed to create file watcher: {}", e)))?;

        // Watch project files
        for watch_path in &self.config.watch_paths {
            watcher.watch(&watch_path, RecursiveMode::Recursive)
                .map_err(|e| crate::Error::Other(format!("Failed to watch path: {}", e)))?;
        }

        let project_config = self.project_config.clone();
        let is_running = self.is_running.clone();
        let project_root = self.config.project_root.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                match rx.try_recv() {
                    Ok(Ok(event)) => {
                        if let Err(e) = Self::handle_file_change(event, &project_config, &project_root).await {
                            CliUtils::error(&format!("Hot reload error: {}", e));
                        }
                    }
                    Ok(Err(e)) => {
                        CliUtils::error(&format!("File watcher error: {}", e));
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        });

        CliUtils::info("Hot reload enabled - watching for file changes");
        Ok(())
    }

    /// Handle file change events
    async fn handle_file_change(
        event: Event,
        project_config: &Arc<RwLock<ProjectConfig>>,
        project_root: &Path,
    ) -> Result<()> {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in event.paths {
                    if let Some(extension) = path.extension() {
                        match extension.to_str() {
                            Some("rs") => {
                                CliUtils::info(&format!("Rust file changed: {}", path.display()));
                                Self::rebuild_project(project_root).await?;
                            }
                            Some("toml") if path.file_name() == Some(std::ffi::OsStr::new("lumos.toml")) => {
                                CliUtils::info("Configuration changed - reloading");
                                Self::reload_config(project_config, &path).await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    CliUtils::warning(&format!("File removed: {}", path.display()));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Rebuild the project
    async fn rebuild_project(project_root: &Path) -> Result<()> {
        CliUtils::progress("Rebuilding project...");
        
        match CliUtils::execute_command("cargo", &["check"], Some(project_root)).await {
            Ok(_) => {
                CliUtils::success("Project rebuilt successfully");
            }
            Err(e) => {
                CliUtils::error(&format!("Build failed: {}", e));
            }
        }
        
        Ok(())
    }

    /// Reload project configuration
    async fn reload_config(
        project_config: &Arc<RwLock<ProjectConfig>>,
        config_path: &Path,
    ) -> Result<()> {
        match CliUtils::load_config(config_path) {
            Ok(new_config) => {
                let mut config = project_config.write().await;
                *config = new_config;
                CliUtils::success("Configuration reloaded");
            }
            Err(e) => {
                CliUtils::error(&format!("Failed to reload config: {}", e));
            }
        }
        Ok(())
    }

    /// Run the main server loop
    async fn run_server_loop(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(1));
        
        while *self.is_running.read().await {
            interval.tick().await;
            
            // Perform periodic tasks
            self.perform_health_checks().await?;
            
            // Check for shutdown signal
            if tokio::signal::ctrl_c().await.is_ok() {
                CliUtils::info("Shutdown signal received");
                break;
            }
        }
        
        self.shutdown().await?;
        Ok(())
    }

    /// Perform health checks
    async fn perform_health_checks(&self) -> Result<()> {
        // Check if project is still valid
        let config_path = self.config.project_root.join("lumos.toml");
        if !config_path.exists() {
            CliUtils::warning("Project configuration not found");
            return Ok(());
        }

        // Check if dependencies are up to date
        // This is a simplified check - in a real implementation,
        // we would check tool versions, etc.
        
        Ok(())
    }

    /// Shutdown the server
    async fn shutdown(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }
        
        CliUtils::info("Development server stopped");
        Ok(())
    }
}

/// Start the development server
pub async fn start_dev_server(
    port: u16,
    hot_reload: bool,
    debug: bool,
    project_root: &Path,
) -> Result<()> {
    // Load project configuration
    let config_path = project_root.join("lumos.toml");
    let project_config = CliUtils::load_config(&config_path)?;

    // Set up watch paths
    let mut watch_paths = vec![
        project_root.join("src"),
        project_root.join("lumos.toml"),
    ];

    // Add additional paths if they exist
    if project_root.join("examples").exists() {
        watch_paths.push(project_root.join("examples"));
    }
    if project_root.join("tests").exists() {
        watch_paths.push(project_root.join("tests"));
    }

    let dev_config = DevServerConfig {
        port,
        hot_reload,
        debug,
        project_root: project_root.to_path_buf(),
        watch_paths,
        ignore_patterns: vec![
            "target".to_string(),
            ".git".to_string(),
            "node_modules".to_string(),
            ".lumos/cache".to_string(),
        ],
    };

    let server = DevServer::new(dev_config, project_config);
    server.start().await?;

    Ok(())
}

/// Development server utilities
pub struct DevServerUtils;

impl DevServerUtils {
    /// Check if a path should be ignored
    pub fn should_ignore_path(path: &Path, ignore_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Get file extension
    pub fn get_file_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Check if file is a source file
    pub fn is_source_file(path: &Path) -> bool {
        match Self::get_file_extension(path).as_deref() {
            Some("rs") | Some("toml") | Some("yaml") | Some("yml") | Some("json") => true,
            _ => false,
        }
    }

    /// Format file size
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Get project statistics
    pub async fn get_project_stats(project_root: &Path) -> Result<ProjectStats> {
        let mut stats = ProjectStats::default();
        
        // Count source files
        if let Ok(entries) = std::fs::read_dir(project_root.join("src")) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && Self::is_source_file(&entry.path()) {
                        stats.source_files += 1;
                        stats.total_size += metadata.len();
                    }
                }
            }
        }
        
        // Count test files
        if let Ok(entries) = std::fs::read_dir(project_root.join("tests")) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && Self::is_source_file(&entry.path()) {
                        stats.test_files += 1;
                    }
                }
            }
        }
        
        Ok(stats)
    }
}

/// Project statistics
#[derive(Debug, Default)]
pub struct ProjectStats {
    pub source_files: u32,
    pub test_files: u32,
    pub total_size: u64,
    pub last_modified: Option<std::time::SystemTime>,
}

impl ProjectStats {
    /// Display formatted statistics
    pub fn display(&self) {
        println!("\n📊 Project Statistics:");
        println!("  Source files: {}", self.source_files);
        println!("  Test files: {}", self.test_files);
        println!("  Total size: {}", DevServerUtils::format_file_size(self.total_size));
        
        if let Some(modified) = self.last_modified {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                println!("  Last modified: {} seconds ago", duration.as_secs());
            }
        }
    }
}
