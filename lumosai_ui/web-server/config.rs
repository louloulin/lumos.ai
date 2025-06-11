use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    /// Server port
    pub port: u16,
    /// Maximum upload size in MB
    pub max_upload_size_mb: usize,
    /// Application version
    pub version: String,
    /// Development mode
    pub development_mode: bool,
    /// Auto open browser
    pub auto_open_browser: bool,
    /// Static files directory
    pub static_dir: String,
    /// Enable hot reload
    pub hot_reload: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Config {
        let port: u16 = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let version: String = env::var("VERSION")
            .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

        let max_upload_size_mb: usize = env::var("MAX_UPLOAD_SIZE_MB")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let development_mode = env::var("DEVELOPMENT_MODE").is_ok() 
            || cfg!(debug_assertions);

        let auto_open_browser = env::var("AUTO_OPEN_BROWSER")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(development_mode);

        let static_dir = env::var("STATIC_DIR")
            .unwrap_or_else(|_| "static".to_string());

        let hot_reload = env::var("HOT_RELOAD")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(development_mode);

        Config {
            port,
            max_upload_size_mb,
            version,
            development_mode,
            auto_open_browser,
            static_dir,
            hot_reload,
        }
    }

    /// Check if we're running in development mode
    pub fn is_development(&self) -> bool {
        self.development_mode
    }

    /// Get the base URL for the application
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Get the static files URL prefix
    pub fn static_url_prefix(&self) -> &str {
        "/static"
    }
}
