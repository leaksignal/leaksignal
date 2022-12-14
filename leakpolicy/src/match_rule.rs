use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};

use anyhow::Result;
use regex::Regex;
use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
enum Mode {
    Regex(Regex),
    Raw(String),
    Except(String),
    ExceptRegex(Regex),
}

impl AsRef<str> for Mode {
    fn as_ref(&self) -> &str {
        match self {
            Mode::Regex(x) => x.as_str(),
            Mode::Raw(x) => x,
            Mode::Except(x) => x,
            Mode::ExceptRegex(x) => x.as_str(),
        }
    }
}

impl PartialEq for Mode {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Mode {}

impl Hash for Mode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

/// Optionally prefixed matching component for string matching.
/// Rules:
/// `regex:` beginning strings are parsed as regex. Automatically anchored to beginning and end of input.
/// `raw:` or unprefixed strings are parsed as raw strings.
/// `except:` meaningless on it's own, but can be used to ignore literal matches from MatchRules in the same context.
/// `except_regex:` Same as `except`, but uses regex.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MatchRule {
    mode: Mode,
}

impl<'de> Deserialize<'de> for MatchRule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(|e| {
            serde::de::Error::invalid_value(
                Unexpected::Str(&raw),
                &format!("invalid PathGlob: {}", e).deref(),
            )
        })
    }
}

impl Serialize for MatchRule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

fn anchor_regex(input: &str) -> Result<Regex> {
    // this can be tricked using escaped dollar signs, but that's acceptable.
    if input.starts_with('^') && input.ends_with('$') {
        Ok(Regex::new(input)?)
    } else if input.starts_with('^') {
        Ok(Regex::new(&format!("{input}$"))?)
    } else if input.ends_with('$') {
        Ok(Regex::new(&format!("^{input}"))?)
    } else {
        Ok(Regex::new(&format!("^{input}$"))?)
    }
}

impl FromStr for MatchRule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(s) = s.strip_prefix("raw:") {
            Ok(Self {
                mode: Mode::Raw(s.to_string()),
            })
        } else if let Some(s) = s.strip_prefix("regex:") {
            Ok(Self {
                mode: Mode::Regex(anchor_regex(s)?),
            })
        } else if let Some(s) = s.strip_prefix("except:") {
            Ok(Self {
                mode: Mode::Except(s.to_string()),
            })
        } else if let Some(s) = s.strip_prefix("except_regex:") {
            Ok(Self {
                mode: Mode::ExceptRegex(anchor_regex(s)?),
            })
        } else {
            Ok(Self {
                mode: Mode::Raw(s.to_string()),
            })
        }
    }
}

impl fmt::Display for MatchRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.mode {
            Mode::Regex(x) => write!(f, "regex:{}", x.as_str())?,
            Mode::Raw(x)
                if x.starts_with("regex:")
                    || x.starts_with("except:")
                    || x.starts_with("except_regex:") =>
            {
                write!(f, "raw:{}", x)?
            }
            Mode::Raw(x) => write!(f, "{}", x)?,
            Mode::Except(x) => write!(f, "except:{}", x)?,
            Mode::ExceptRegex(x) => write!(f, "except_regex:{}", x.as_str())?,
        }
        Ok(())
    }
}

impl AsRef<MatchRule> for MatchRule {
    fn as_ref(&self) -> &MatchRule {
        self
    }
}

impl MatchRule {
    /// Returns `true` if this MatchRule individually matches the given text. `except`/`except_regex` are matched as if they were not `except`-class.
    pub fn matches(&self, input: &str) -> bool {
        match &self.mode {
            Mode::Regex(x) | Mode::ExceptRegex(x) => x.is_match(input),
            Mode::Except(x) | Mode::Raw(x) => input == x,
        }
    }

    /// Returns `true` if this is a negative MatchRule.
    pub fn is_exception(&self) -> bool {
        matches!(&self.mode, Mode::Except(_) | Mode::ExceptRegex(_))
    }

    /// Matches a slice of MatchRules, properly evaluating negative rules.
    pub fn match_all<A: AsRef<MatchRule>>(input: &str, match_rules: &[A]) -> bool {
        let mut matched = None::<bool>;
        for rule in match_rules {
            let item = rule.as_ref();
            if matched == Some(true) && !item.is_exception() {
                continue;
            }
            if item.matches(input) {
                if item.is_exception() {
                    return false;
                }
                matched = Some(true);
            } else if !item.is_exception() {
                matched = Some(false);
            }
        }
        matched.unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_rules() {
        let raw: MatchRule = "test".parse().unwrap();
        let raw2: MatchRule = "raw:test".parse().unwrap();
        let regex: MatchRule = "regex:test".parse().unwrap();
        let regex2: MatchRule = "regex:t.*t".parse().unwrap();
        let except: MatchRule = "except:tast".parse().unwrap();
        let except2: MatchRule = "except:tset".parse().unwrap();
        let except_regex: MatchRule = "except_regex:ta.*t".parse().unwrap();
        let except_regex2: MatchRule = "except_regex:tb.*t".parse().unwrap();

        assert!(raw.matches("test"));
        assert!(!raw.matches("test2"));
        assert!(!raw.matches("2test"));
        assert!(raw2.matches("test"));
        assert!(!raw2.matches("test2"));
        assert!(!raw2.matches("2test"));
        assert!(regex.matches("test"));
        assert!(!regex.matches("test2"));
        assert!(!regex.matches("2test"));
        assert!(regex2.matches("test"));
        assert!(!regex2.matches("test2"));
        assert!(!regex2.matches("2test"));

        assert!(regex2.matches("tast"));
        assert!(!regex2.matches("taste"));

        assert!(MatchRule::match_all("test", &[&regex2, &except]));
        assert!(!MatchRule::match_all("tast", &[&regex2, &except]));
        assert!(!MatchRule::match_all("tset", &[&except2, &regex2, &except]));
        assert!(!MatchRule::match_all("tasst", &[&regex2, &except_regex]));
        assert!(MatchRule::match_all("tbest", &[&regex2, &except_regex]));
        assert!(!MatchRule::match_all("tbsst", &[&regex2, &except_regex2]));
        assert!(MatchRule::match_all("taest", &[&regex2, &except_regex2]));
    }
}
