pub mod get_herd;
pub mod update;

use crate::domain;
use crate::domain::time::{DateTime, Duration};
use crate::errors::{Error, Result};
use anyhow::anyhow;

pub trait UpdateHandler {
    async fn handle(&self) -> Result<()>;
}

pub trait GetHerdHandler {
    fn get_herd(&self) -> Result<Herd>;
}

pub trait Rancher {
    async fn update(&self) -> Result<()>;
    fn get_cow_statuses(&self) -> Result<Vec<domain::CowStatus>>;
}

pub trait Metrics {
    fn record_application_handler_call(
        &self,
        handler_name: &str,
        result: ApplicationHandlerCallResult,
        duration: Duration,
    );

    fn update_herd_numbers(&self, herd: &Herd);
}

pub enum ApplicationHandlerCallResult {
    Ok,
    Error,
}

impl<T> Rancher for domain::Rancher<T>
where
    T: domain::Inventory,
{
    async fn update(&self) -> Result<()> {
        self.update().await
    }

    fn get_cow_statuses(&self) -> Result<Vec<domain::CowStatus>> {
        self.get_cow_statuses()
    }
}

pub struct Herd {
    cows: Vec<Cow>,
}

impl Herd {
    pub fn cows(&self) -> &[Cow] {
        &self.cows
    }
}

impl TryFrom<Vec<domain::CowStatus>> for Herd {
    type Error = Error;

    fn try_from(value: Vec<domain::CowStatus>) -> Result<Self> {
        let cows: Result<Vec<_>> = value.iter().map(Cow::try_from).collect();
        Ok(Self { cows: cows? })
    }
}

pub struct Cow {
    name: CensoredCow,
    last_seen: Option<DateTime>,
    status: CowStatus,
}

impl Cow {
    pub fn name(&self) -> &CensoredCow {
        &self.name
    }

    pub fn last_seen(&self) -> Option<&DateTime> {
        self.last_seen.as_ref()
    }

    pub fn status(&self) -> &CowStatus {
        &self.status
    }
}

impl TryFrom<&domain::CowStatus> for Cow {
    type Error = Error;

    fn try_from(value: &domain::CowStatus) -> Result<Self> {
        Ok(Self {
            name: CensoredCow::new(value.cow())?,
            last_seen: value.last_seen().cloned(),
            status: CowStatus::new(value),
        })
    }
}

pub struct CensoredCow {
    url: String,
}

impl CensoredCow {
    pub fn new(cow: &domain::Cow) -> Result<Self> {
        let url = cow.url();

        let scheme = url.scheme();
        let host = url
            .host_str()
            .ok_or_else(|| Error::Unknown(anyhow!("no host in url")))?;
        let port = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
        let path = url.path();

        let last_dot_pos = host
            .rfind('.')
            .ok_or_else(|| Error::Unknown(anyhow!("no TLD found in host")))?;
        let (before_tld, tld_with_dot) = host.split_at(last_dot_pos);

        let censored_before: String = before_tld
            .chars()
            .map(|c| if c == '.' { '.' } else { '*' })
            .collect();
        let censored_host = format!("{}{}", censored_before, tld_with_dot);

        // Censor path elements except the final /cow.txt
        let censored_path = if path.ends_with("/cow.txt") && path.len() > 8 {
            // There are path elements before /cow.txt
            let before_cow = &path[..path.len() - 8]; // Remove "/cow.txt"
            let censored_before: String = before_cow
                .chars()
                .map(|c| if c == '/' { '/' } else { '*' })
                .collect();
            format!("{}/cow.txt", censored_before)
        } else {
            path.to_string()
        };

        let censored_url = format!("{}://{}{}{}", scheme, censored_host, port, censored_path);
        Ok(Self { url: censored_url })
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

pub enum CowStatus {
    HappilyGrazing,
    RanAway,
    HaveNotCheckedYet,
}

impl CowStatus {
    pub fn all_variants() -> &'static [CowStatus] {
        &[
            CowStatus::HappilyGrazing,
            CowStatus::RanAway,
            CowStatus::HaveNotCheckedYet,
        ]
    }

    fn new(cow_status: &domain::CowStatus) -> Self {
        if cow_status.last_checked().is_none() {
            return CowStatus::HaveNotCheckedYet;
        }

        let seen_in_last_24h = cow_status
            .last_seen()
            .map(|v| DateTime::now() - v < Duration::new_from_hours(24))
            .unwrap_or(false);
        if seen_in_last_24h {
            return CowStatus::HappilyGrazing;
        }

        CowStatus::RanAway
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CensoredCowTestCase {
        input: &'static str,
        expected: &'static str,
    }

    #[test]
    fn test_censored_cow() {
        let test_cases = vec![
            CensoredCowTestCase {
                input: "https://example.com/cow.txt",
                expected: "https://*******.com/cow.txt",
            },
            CensoredCowTestCase {
                input: "https://www.example.com/cow.txt",
                expected: "https://***.*******.com/cow.txt",
            },
            CensoredCowTestCase {
                input: "https://example.com:8080/cow.txt",
                expected: "https://*******.com:8080/cow.txt",
            },
            CensoredCowTestCase {
                input: "https://api123.example.com/cow.txt",
                expected: "https://******.*******.com/cow.txt",
            },
            CensoredCowTestCase {
                input: "http://example.com/cow.txt",
                expected: "http://*******.com/cow.txt",
            },
            CensoredCowTestCase {
                input: "https://example.com/path/to/cow.txt",
                expected: "https://*******.com/****/**/cow.txt",
            },
        ];

        for test_case in test_cases {
            let cow = domain::Cow::new(test_case.input.to_string()).unwrap();
            let censored = CensoredCow::new(&cow).unwrap();
            assert_eq!(
                censored.url(),
                test_case.expected,
                "Failed for input: {}",
                test_case.input
            );
        }
    }
}
