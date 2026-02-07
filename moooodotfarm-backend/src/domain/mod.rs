pub mod time;

use crate::domain::time::{DateTime, Duration};
use crate::errors::Error;
use crate::errors::Result;
use anyhow::anyhow;
use std::fmt;
use std::fmt::{Display, Formatter};

const COW_BODY: &str = include_str!("../ports/http/static/cow.txt");

const COW_SUFFIX: &str = "/cow.txt";

static CHECK_COW_IF_NOT_CHECKED_FOR_HOURS: u64 = 2;
static CHECK_COW_WHICH_WAS_NEVER_SEEN_IF_NOT_CHECKED_FOR_MINUTES: u64 = 15;

#[derive(Debug, Clone)]
pub struct Cow {
    name: VisibleName,
    character: Character,
    first_seen: Option<DateTime>,
    last_seen: Option<DateTime>,
    last_checked: Option<DateTime>,
}

impl Cow {
    pub fn new(name: VisibleName, character: Character) -> Self {
        Self {
            name,
            character,
            first_seen: None,
            last_seen: None,
            last_checked: None,
        }
    }

    pub fn new_from_history(
        name: VisibleName,
        character: Character,
        first_seen: Option<DateTime>,
        last_seen: Option<DateTime>,
        last_checked: Option<DateTime>,
    ) -> Self {
        Self {
            name,
            character,
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

    pub fn change_character(&mut self, new_character: Character) -> Result<()> {
        if self.character == new_character {
            return Err(Error::Unknown(anyhow!(
                "cow already has the character: {:?}",
                new_character
            )));
        }
        self.character = new_character;
        Ok(())
    }

    pub fn name(&self) -> &VisibleName {
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

impl fmt::Display for Cow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().url)
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

#[derive(Clone)]
pub struct CensoredCow {
    name: Name,
    character: Character,
    first_seen: Option<DateTime>,
    last_seen: Option<DateTime>,
    last_checked: Option<DateTime>,
}

impl CensoredCow {
    pub fn new(cow: &Cow) -> Result<Self> {
        Ok(Self {
            name: Name::new(cow)?,
            character: cow.character().clone(),
            first_seen: cow.first_seen.clone(),
            last_seen: cow.last_seen.clone(),
            last_checked: cow.last_checked.clone(),
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

pub struct CowTxt<'a> {
    content: std::borrow::Cow<'a, str>,
}

impl<'a> CowTxt<'a> {
    pub fn new(content: impl Into<std::borrow::Cow<'a, str>>) -> Result<Self> {
        let content = content.into();
        if !Self::cow_is_present(&content) {
            return Err(Error::CowIsNotPresent(content.into_owned()));
        }

        Ok(Self { content })
    }

    fn cow_is_present(s: &str) -> bool {
        let a: String = s.into();
        let a = Self::trim_trailing_whitespace_from_each_line(a.trim());
        let b = Self::trim_trailing_whitespace_from_each_line(COW_BODY.trim());
        let distance = edit_distance::edit_distance(a, b);
        distance < 100
    }

    fn trim_trailing_whitespace_from_each_line(s: &str) -> String {
        s.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl<'a> Display for CowTxt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
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
            expected_ok: bool,
        }

        let test_cases = vec![
            CowValidationTestCase {
                name: "valid cow",
                input: read_to_string(fixtures::test_file_path("src/ports/http/static/cow.txt"))?,
                expected_ok: true,
            },
            CowValidationTestCase {
                name: "not a cow",
                input: "not a cow".to_string(),
                expected_ok: false,
            },
            CowValidationTestCase {
                name: "cow with no trailing whitespace",
                input: read_to_string(fixtures::test_file_path(
                    "src/domain/testdata/cow_with_no_trailing_whitespace.txt",
                ))?,
                expected_ok: true,
            },
        ];

        for test_case in test_cases {
            let actual = CowTxt::new(&test_case.input);
            match actual {
                Ok(_) => {
                    assert!(
                        test_case.expected_ok,
                        "Expected test case to fail but it succeeded: {}",
                        test_case.name
                    );
                }
                Err(err) => {
                    println!("Error: {}", err);
                    assert!(
                        !test_case.expected_ok,
                        "Expected test case to succeed but it failed: {}",
                        test_case.name
                    );
                }
            }
        }

        Ok(())
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
            let cow = Cow::new(visible_name, test_case.character);
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
