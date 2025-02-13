use anyhow::Context;
use std::os::unix::process::CommandExt;
use anyhow::Result;
use anyhow::Error;
use clap::Parser;
use directories::ProjectDirs;
use std::process::Command;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Parser)]
struct Args {
    url: Url
}

#[derive(Debug, Copy, Clone, Default, Deserialize)]
enum Browser {
    Chrome,
    #[default]
    Firefox,
}

impl Browser {
    fn command(&self, url: &Url) -> Command {
        let mut cmd = match self {
            Self::Chrome => Command::new("chrome"),
            Self::Firefox => Command::new("firefox"),
        };
        cmd.arg(url.to_string());
        cmd
    }
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    default: Browser,
}

impl Config {
    fn route(&self, url: &Url) -> Browser {
        // TODO: actually route based on different rules
        self.default
    }
}


fn main() -> Result<()> {
    let args = Args::parse();
    let project_dirs =
        ProjectDirs::from("ie.vinn", "", "browserdemux").context("while looking for config dir")?;
    let config_path = project_dirs.config_dir().join("config.toml");
    let config: Config = match config_path.exists() {
        false => Default::default(),
        true => {
            let txt = std::fs::read_to_string(&config_path).with_context(||format!("while reading config file '{}'", config_path.display()))?;
            toml::from_str(&txt).
            with_context(||format!("while parsing config file '{}'", config_path.display()))?
        }
    };

    Err(Error::from(config.route(&args.url).command(&args.url).exec()).context("while execing browser"))
}
