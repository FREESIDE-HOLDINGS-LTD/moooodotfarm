pub mod time;

use crate::domain::time::{DateTime, Duration};
use crate::errors::Error;
use crate::errors::Result;
use anyhow::anyhow;
use std::fmt::{Display, Formatter};

const COW_BODY: &str = include_str!("../ports/http/static/cow.txt");

const COW_SUFFIX: &str = "/cow.txt";

static CHECK_COW_IF_NOT_CHECKED_FOR_HOURS: u64 = 2;
static CHECK_COW_WHICH_WAS_NEVER_SEEN_IF_NOT_CHECKED_FOR_MINUTES: u64 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cow {
    name: VisibleName,
    character: Character,
}

impl Cow {
    pub fn new(name: VisibleName, character: Character) -> Result<Self> {
        Ok(Cow { name, character })
    }

    pub fn name(&self) -> &VisibleName {
        &self.name
    }

    pub fn character(&self) -> &Character {
        &self.character
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleName {
    url: url::Url,
}

impl VisibleName {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let url = url::Url::parse(&s.into())?;
        if !url.path().ends_with(COW_SUFFIX) {
            return Err(Error::Unknown(anyhow!(
                "cow must have a tail and end with '{}'",
                COW_SUFFIX
            )));
        }
        Ok(Self { url })
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

impl Display for Cow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.url())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensoredName {
    url: String,
}

impl CensoredName {
    pub fn new(cow: &Cow) -> Result<Self> {
        if cow.character == Character::Brave {
            return Ok(Self {
                url: cow.name().url().to_string(),
            });
        }

        let url = cow.name().url();

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Visible(VisibleName),
    Censored(CensoredName),
}

impl Name {
    pub fn new(cow: &Cow) -> Result<Self> {
        match cow.character() {
            Character::Brave => Ok(Name::Visible(cow.name().clone())),
            Character::Shy => Ok(Name::Censored(CensoredName::new(cow)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Character {
    Brave,
    Shy,
}

pub fn cow_is_present(s: impl Into<String>) -> bool {
    let a: String = s.into();
    let a = trim_trailing_whitespace_from_each_line(a.trim());
    let b = trim_trailing_whitespace_from_each_line(COW_BODY.trim());
    let distance = edit_distance::edit_distance(a, b);
    distance < 100
}
fn trim_trailing_whitespace_from_each_line(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct Herd {
    cows: Vec<Cow>,
}

impl Herd {
    pub fn new(mut cows: Vec<Cow>) -> Result<Self> {
        if cows.is_empty() {
            return Err(Error::Unknown(anyhow!("all the cows have escaped")));
        }
        cows.sort_by(|a, b| a.name().url().cmp(b.name().url()));

        for i in 1..cows.len() {
            if cows[i - 1].name().url() == cows[i].name().url() {
                return Err(Error::Unknown(anyhow!("duplicate cow found {}", cows[i])));
            }
        }

        Ok(Herd { cows })
    }

    pub fn cows(&self) -> &[Cow] {
        &self.cows
    }
}

#[derive(Clone)]
pub struct CowStatus {
    name: VisibleName,
    first_seen: Option<DateTime>,
    last_seen: Option<DateTime>,
    last_checked: Option<DateTime>,
}

impl CowStatus {
    pub fn new(name: VisibleName) -> Self {
        Self {
            name,
            first_seen: None,
            last_seen: None,
            last_checked: None,
        }
    }

    pub fn new_from_history(
        name: VisibleName,
        first_seen: Option<DateTime>,
        last_seen: Option<DateTime>,
        last_checked: Option<DateTime>,
    ) -> Self {
        Self {
            name,
            first_seen,
            last_seen,
            last_checked,
        }
    }

    pub fn should_check(&self) -> bool {
        if let Some(last_checked) = &self.last_checked {
            let duration = if self.first_seen.is_none() {
                Duration::new_from_minutes(
                    CHECK_COW_WHICH_WAS_NEVER_SEEN_IF_NOT_CHECKED_FOR_MINUTES,
                )
            } else {
                Duration::new_from_hours(CHECK_COW_IF_NOT_CHECKED_FOR_HOURS)
            };
            return &DateTime::now() - last_checked > duration;
        }
        true
    }

    pub fn mark_as_ok(&mut self) {
        let now = DateTime::now();

        if self.first_seen.is_none() {
            self.first_seen = Some(now.clone());
        }

        self.last_seen = Some(now.clone());
        self.last_checked = Some(now.clone());
    }

    pub fn mark_as_missing(&mut self) {
        let now = DateTime::now();
        self.last_checked = Some(now.clone());
    }

    pub fn name(&self) -> &VisibleName {
        &self.name
    }

    pub fn first_seen(&self) -> Option<&DateTime> {
        self.first_seen.as_ref()
    }

    pub fn last_seen(&self) -> Option<&DateTime> {
        self.last_seen.as_ref()
    }

    pub fn last_checked(&self) -> Option<&DateTime> {
        self.last_checked.as_ref()
    }
}

#[derive(Clone)]
pub struct CensoredCowStatus {
    name: Name,
    character: Character,
    first_seen: Option<DateTime>,
    last_seen: Option<DateTime>,
    last_checked: Option<DateTime>,
}

impl CensoredCowStatus {
    pub fn new(cow: &Cow, cow_status: &CowStatus) -> Result<Self> {
        Ok(Self {
            name: Name::new(cow)?,
            character: cow.character().clone(),
            first_seen: cow_status.first_seen.clone(),
            last_seen: cow_status.last_seen.clone(),
            last_checked: cow_status.last_checked.clone(),
        })
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn character(&self) -> &Character {
        &self.character
    }

    pub fn first_seen(&self) -> Option<&DateTime> {
        self.first_seen.as_ref()
    }

    pub fn last_seen(&self) -> Option<&DateTime> {
        self.last_seen.as_ref()
    }

    pub fn last_checked(&self) -> Option<&DateTime> {
        self.last_checked.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::fs::read_to_string;

    #[test]
    fn cow_validation_works() -> Result<()> {
        struct CowValidationTestCase {
            name: &'static str,
            input: String,
            expected: bool,
        }

        let test_cases = vec![
            CowValidationTestCase {
                name: "valid cow",
                input: read_to_string(fixtures::test_file_path("src/ports/http/static/cow.txt"))?,
                expected: true,
            },
            CowValidationTestCase {
                name: "not a cow",
                input: "not a cow".to_string(),
                expected: false,
            },
            CowValidationTestCase {
                name: "cow with no trailing whitespace",
                input: read_to_string(fixtures::test_file_path(
                    "src/domain/testdata/cow_with_no_trailing_whitespace.txt",
                ))?,
                expected: true,
            },
        ];

        for test_case in test_cases {
            let actual = cow_is_present(&test_case.input);
            assert_eq!(
                actual, test_case.expected,
                "Failed for test case: {}",
                test_case.name
            );
        }

        Ok(())
    }

    #[test]
    fn duplicate_cows_in_herd_are_detected_even_if_they_are_not_next_to_each_other() {
        let cow1 = Cow::new(
            VisibleName::new("http://example.com/cow.txt").unwrap(),
            Character::Brave,
        )
        .unwrap();
        let cow2 = Cow::new(
            VisibleName::new("http://example.org/cow.txt").unwrap(),
            Character::Brave,
        )
        .unwrap();
        let cow3 = Cow::new(
            VisibleName::new("http://example.com/cow.txt").unwrap(),
            Character::Brave,
        )
        .unwrap();

        let herd_result = Herd::new(vec![cow1, cow2, cow3]);
        assert!(herd_result.is_err());
    }

    #[test]
    fn test_censored_name() {
        struct CensoredNameTestCase {
            input: &'static str,
            character: Character,
            expected: &'static str,
        }

        let test_cases = vec![
            CensoredNameTestCase {
                input: "https://example.com/cow.txt",
                character: Character::Brave,
                expected: "https://example.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://example.com/cow.txt",
                character: Character::Shy,
                expected: "https://*******.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://www.example.com/cow.txt",
                character: Character::Brave,
                expected: "https://www.example.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://www.example.com/cow.txt",
                character: Character::Shy,
                expected: "https://***.*******.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://example.com:8080/cow.txt",
                character: Character::Brave,
                expected: "https://example.com:8080/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://example.com:8080/cow.txt",
                character: Character::Shy,
                expected: "https://*******.com:8080/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://api123.example.com/cow.txt",
                character: Character::Brave,
                expected: "https://api123.example.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://api123.example.com/cow.txt",
                character: Character::Shy,
                expected: "https://******.*******.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "http://example.com/cow.txt",
                character: Character::Brave,
                expected: "http://example.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "http://example.com/cow.txt",
                character: Character::Shy,
                expected: "http://*******.com/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://example.com/path/to/cow.txt",
                character: Character::Brave,
                expected: "https://example.com/path/to/cow.txt",
            },
            CensoredNameTestCase {
                input: "https://example.com/path/to/cow.txt",
                character: Character::Shy,
                expected: "https://*******.com/****/**/cow.txt",
            },
        ];

        for test_case in test_cases {
            let visible_name = VisibleName::new(test_case.input.to_string()).unwrap();
            let cow = Cow::new(visible_name, test_case.character).unwrap();
            let name = Name::new(&cow).unwrap();
            let actual_url = match name {
                Name::Visible(v) => v.url().to_string(),
                Name::Censored(c) => c.url().to_string(),
            };
            assert_eq!(
                actual_url, test_case.expected,
                "Failed for input: {}",
                test_case.input
            );
        }
    }
}
