use std::{env, net::Ipv4Addr, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: Ipv4Addr,
    pub port: u16,
    pub web_dist_dir: PathBuf,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("AMI_OKAY_HOST")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(Ipv4Addr::new(127, 0, 0, 1));

        let port = env::var("AMI_OKAY_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8787);

        let web_dist_dir = env::var("AMI_OKAY_WEB_DIST")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("../web/dist"));
        let database_url = env::var("AMI_OKAY_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://DB/eyes-on-me.db".to_string());

        Self {
            host,
            port,
            web_dist_dir,
            database_url,
        }
    }
}
