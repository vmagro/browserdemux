use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use clap::Parser;
use directories::ProjectDirs;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Parser)]
struct Args {
    url: Url,
}

#[derive(Debug, Copy, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Browser {
    #[default]
    Firefox,
    GoogleChrome,
}

impl Browser {
    fn command(&self, url: &Url) -> Command {
        let mut cmd = match self {
            Self::Firefox => Command::new("firefox"),
            Self::GoogleChrome => Command::new("google-chrome"),
        };
        cmd.arg(url.to_string());
        cmd
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    to: Browser,
    #[serde(rename = "match")]
    matcher: Matcher,
}

impl Rule {
    fn matches(&self, url: &Url) -> bool {
        self.matcher.matches(url)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Matcher {
    Authority(String),
    Domain(String),
}

impl Matcher {
    fn matches(&self, url: &Url) -> bool {
        match self {
            Self::Authority(authority) => authority == url.authority(),
            Self::Domain(domain) => {
                if url.authority() == domain {
                    true
                } else {
                    url.authority().ends_with(&format!(".{domain}"))
                }
            }
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    default: Browser,
    #[serde(rename = "rule", default)]
    rules: Vec<Rule>,
}

impl Config {
    fn route(&self, url: &Url) -> Browser {
        for rule in &self.rules {
            if rule.matches(url) {
                return rule.to;
            }
        }
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
            let txt = std::fs::read_to_string(&config_path).with_context(|| {
                format!("while reading config file '{}'", config_path.display())
            })?;
            toml::from_str(&txt)
                .with_context(|| format!("while parsing config file '{}'", config_path.display()))?
        }
    };

    Err(
        Error::from(config.route(&args.url).command(&args.url).exec())
            .context("while execing browser"),
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn match_authority() {
        let m = Matcher::Authority("facebook.com".to_owned());
        assert!(m.matches(&Url::from_str("https://facebook.com").expect("valid url")));
        assert!(!m.matches(&Url::from_str("https://thefacebook.com").expect("valid url")));
        assert!(!m.matches(&Url::from_str("https://the.facebook.com").expect("valid url")));
    }

    #[test]
    fn match_domain() {
        let m = Matcher::Domain("facebook.com".to_owned());
        assert!(m.matches(&Url::from_str("https://facebook.com").expect("valid url")));
        assert!(!m.matches(&Url::from_str("https://thefacebook.com").expect("valid url")));
        assert!(m.matches(&Url::from_str("https://the.facebook.com").expect("valid url")));
        let m = Matcher::Domain("the.facebook.com".to_owned());
        assert!(!m.matches(&Url::from_str("https://facebook.com").expect("valid url")));
        assert!(!m.matches(&Url::from_str("https://thefacebook.com").expect("valid url")));
        assert!(m.matches(&Url::from_str("https://the.facebook.com").expect("valid url")));
        assert!(m.matches(&Url::from_str("https://drop.the.facebook.com").expect("valid url")));
    }
}
