pub mod database;

use crate::app;
use crate::app::{ApplicationHandlerCallResult, Herd};
use crate::config::{Config, Environment};
use crate::domain;
use crate::domain::time::Duration;
use crate::domain::{Cow, CowTxt, VisibleName};
use crate::errors::Result;
use anyhow::anyhow;
use prometheus::{CounterVec, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry, labels};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct ConfigLoader {
    path: PathBuf,
}

impl ConfigLoader {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<Config> {
        let content = fs::read_to_string(&self.path)?;
        let transport: TomlConfig = toml::from_str(&content)?;
        Config::try_from(transport)
    }
}

#[derive(Deserialize)]
struct TomlConfig {
    address: String,
    environment: String,
    database_path: String,
    cows: Vec<TomlCow>,
}

#[derive(Deserialize)]
struct TomlCow {
    name: String,
    character: String,
}

impl TryFrom<TomlConfig> for Config {
    type Error = crate::errors::Error;

    fn try_from(value: TomlConfig) -> std::result::Result<Self, Self::Error> {
        let cows = value
            .cows
            .into_iter()
            .map(|toml_cow| {
                let character = toml_cow.character.try_into()?;
                let name = domain::VisibleName::new(toml_cow.name)?;
                Cow::new(name, character)
            })
            .collect::<Result<Vec<_>>>()?;
        Config::new(
            value.address,
            value.environment.try_into()?,
            value.database_path,
            cows,
        )
    }
}

impl TryFrom<String> for Environment {
    type Error = crate::errors::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "production" => Ok(Environment::Production),
            "development" => Ok(Environment::Development),
            other => Err(anyhow!("invalid environment: {}", other).into()),
        }
    }
}

impl TryFrom<String> for crate::domain::Character {
    type Error = crate::errors::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "brave" => Ok(crate::domain::Character::Brave),
            "shy" => Ok(crate::domain::Character::Shy),
            other => Err(anyhow!("invalid character: {}", other).into()),
        }
    }
}

#[derive(Clone)]
pub struct Metrics {
    registry: Registry,

    metric_application_handler_calls_counter: CounterVec,
    metric_application_handler_calls_histogram: HistogramVec,
    metric_herd_numbers: GaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new_custom(Some("moooodotfarm".into()), None)?;

        let metric_application_handler_calls_counter = CounterVec::new(
            Opts::new(
                "application_handler_calls_counter",
                "application handler calls counter",
            ),
            &["handler_name", "result"],
        )?;
        registry.register(Box::new(metric_application_handler_calls_counter.clone()))?;

        let metric_application_handler_calls_histogram = HistogramVec::new(
            HistogramOpts::new(
                "application_handler_calls_histogram",
                "application handler calls durations",
            ),
            &["handler_name", "result"],
        )?;
        registry.register(Box::new(metric_application_handler_calls_histogram.clone()))?;

        let metric_herd_numbers = GaugeVec::new(
            Opts::new("herd_numbers", "number of cows grouped by status"),
            &["status"],
        )?;
        registry.register(Box::new(metric_herd_numbers.clone()))?;

        Ok(Self {
            registry,

            metric_application_handler_calls_counter,
            metric_application_handler_calls_histogram,
            metric_herd_numbers,
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl app::Metrics for Metrics {
    fn record_application_handler_call(
        &self,
        handler_name: &str,
        result: ApplicationHandlerCallResult,
        duration: Duration,
    ) {
        let labels = labels! {
            "handler_name" => handler_name,
            "result" => match result {
                ApplicationHandlerCallResult::Ok => "ok",
                ApplicationHandlerCallResult::Error => "error"
            },
        };

        self.metric_application_handler_calls_counter
            .with(&labels)
            .inc();

        self.metric_application_handler_calls_histogram
            .with(&labels)
            .observe(duration.as_seconds());
    }

    fn update_herd_numbers(&self, herd: &Herd) {
        let mut counts: HashMap<&str, i64> = HashMap::new();

        for cow in herd.cows() {
            let status_key = cow_status_as_str(cow.status());
            *counts.entry(status_key).or_insert(0) += 1;
        }

        for status in app::CowStatus::all_variants() {
            let status_str = cow_status_as_str(status);
            let count = counts.get(status_str).copied().unwrap_or(0);

            self.metric_herd_numbers
                .with(&labels! { "status" => status_str })
                .set(count as f64);
        }
    }
}

fn cow_status_as_str(status: &app::CowStatus) -> &'static str {
    match status {
        app::CowStatus::HappilyGrazing => "happily_grazing",
        app::CowStatus::RanAway => "ran_away",
        app::CowStatus::HaveNotCheckedYet => "have_not_checked_yet",
    }
}

#[derive(Clone)]
pub struct CowTxtDownloader {}

impl Default for CowTxtDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl CowTxtDownloader {
    pub fn new() -> Self {
        Self {}
    }
}

impl app::CowTxtDownloader for CowTxtDownloader {
    async fn download(&self, name: &VisibleName) -> Result<CowTxt<'_>> {
        let cow_body = reqwest::get(name.url().to_string()).await?.text().await?;
        CowTxt::new(cow_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::VisibleName;
    use crate::fixtures;

    #[test]
    fn loads_config_from_file_successfully() -> Result<()> {
        use crate::domain::Character;
        let expected_config = Config::new(
            "0.0.0.0:8080",
            Environment::Development,
            "/moooodotfarm.db",
            vec![Cow::new(
                VisibleName::new("https://moooo.farm/cow.txt")?,
                Character::Brave,
            )?],
        )?;
        let loader = ConfigLoader::new(fixtures::test_file_path(
            "src/adapters/testdata/config.toml",
        ));
        let config = loader.load()?;
        assert_eq!(expected_config, config);
        Ok(())
    }
}
