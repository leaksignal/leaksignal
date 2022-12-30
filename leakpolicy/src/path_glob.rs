use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use anyhow::Result;
use regex::Regex;
use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
enum GlobComponent {
    AnyOne,
    AnyMany,
    Regex(String, Regex),
    Contains(String),
    Prefix(String),
    Suffix(String),
    Literal(String),
}

impl PartialEq for GlobComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Regex(l0, _), Self::Regex(r0, _)) => l0 == r0,
            (Self::Contains(l0), Self::Contains(r0)) => l0 == r0,
            (Self::Prefix(l0), Self::Prefix(r0)) => l0 == r0,
            (Self::Suffix(l0), Self::Suffix(r0)) => l0 == r0,
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for GlobComponent {}

impl Hash for GlobComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            GlobComponent::Regex(s, _)
            | GlobComponent::Contains(s)
            | GlobComponent::Prefix(s)
            | GlobComponent::Suffix(s)
            | GlobComponent::Literal(s) => s.hash(state),
            _ => (),
        }
    }
}

impl GlobComponent {
    fn globish(&self) -> bool {
        match self {
            GlobComponent::AnyOne => false,
            GlobComponent::AnyMany => false,
            GlobComponent::Regex(_, _) => true,
            GlobComponent::Contains(_) => true,
            GlobComponent::Prefix(_) => true,
            GlobComponent::Suffix(_) => true,
            GlobComponent::Literal(_) => false,
        }
    }

    fn matches(&self, target: &str) -> bool {
        match self {
            GlobComponent::AnyOne => true,
            GlobComponent::AnyMany => true,
            GlobComponent::Regex(_, r) => match r.find(target) {
                None => false,
                Some(matching) => matching.start() == 0 && matching.end() == target.len(),
            },
            GlobComponent::Contains(s) => target.contains(&s[1..s.len() - 1]),
            GlobComponent::Prefix(s) => target.starts_with(&s[..s.len() - 1]),
            GlobComponent::Suffix(s) => target.ends_with(&s[1..]),
            GlobComponent::Literal(s) => target == s,
        }
    }
}

impl AsRef<str> for GlobComponent {
    fn as_ref(&self) -> &str {
        match self {
            GlobComponent::AnyOne => "*",
            GlobComponent::AnyMany => "**",
            GlobComponent::Regex(s, _) => s,
            GlobComponent::Contains(s) => s,
            GlobComponent::Prefix(s) => s,
            GlobComponent::Suffix(s) => s,
            GlobComponent::Literal(s) => s,
        }
    }
}

/// Path component based (split by '/') matcher with the following patterns:
/// `*`: match any single component
/// `**`: match any number of arbitrary components (lazy)
/// `#<regex>`: matches the regex against the component (must fully match)
/// `*<string>*`: the `string` must be contained somewhere in the component
/// `*<string>`: the `string` must be at the end of the component
/// `<string>*`: the `string` must be at the start of the component
/// `<string>`: the `string` must exactly the component
///
/// These components are strung together by `/` during both parsing of the `PathGlob` and matching against a string.
/// See tests for examples.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PathGlob {
    components: Vec<GlobComponent>,
}

impl PartialOrd for PathGlob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other.components.len().cmp(&self.components.len()) {
            Ordering::Equal => (),
            x => return Some(x),
        }

        let many_glob_count_left = self
            .components
            .iter()
            .filter(|x| matches!(x, GlobComponent::AnyMany))
            .count();
        let many_glob_count_right = other
            .components
            .iter()
            .filter(|x| matches!(x, GlobComponent::AnyMany))
            .count();

        match many_glob_count_left.cmp(&many_glob_count_right) {
            Ordering::Equal => (),
            x => return Some(x),
        }

        let glob_count_left = self
            .components
            .iter()
            .filter(|x| matches!(x, GlobComponent::AnyOne))
            .count();
        let glob_count_right = other
            .components
            .iter()
            .filter(|x| matches!(x, GlobComponent::AnyOne))
            .count();

        match glob_count_left.cmp(&glob_count_right) {
            Ordering::Equal => (),
            x => return Some(x),
        }

        let globish_count_left = self.components.iter().filter(|x| x.globish()).count();
        let globish_count_right = other.components.iter().filter(|x| x.globish()).count();

        match globish_count_left.cmp(&globish_count_right) {
            Ordering::Equal => (),
            x => return Some(x),
        }
        Some(Ordering::Equal)
    }
}

impl Ord for PathGlob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'de> Deserialize<'de> for PathGlob {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(|e| {
            serde::de::Error::invalid_value(
                Unexpected::Str(&raw),
                &&*format!("invalid PathGlob: {}", e),
            )
        })
    }
}

impl Serialize for PathGlob {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl FromStr for PathGlob {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let components = s
            .split('/')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|item| {
                Ok(match item {
                    "*" => GlobComponent::AnyOne,
                    "**" => GlobComponent::AnyMany,
                    s if s.starts_with('#') => {
                        GlobComponent::Regex(s.to_string(), Regex::new(&s[1..])?)
                    }
                    s if s.starts_with('*') && s.ends_with('*') => {
                        GlobComponent::Contains(s.to_string())
                    }
                    s if s.starts_with('*') => GlobComponent::Suffix(s.to_string()),
                    s if s.ends_with('*') => GlobComponent::Prefix(s.to_string()),
                    s => GlobComponent::Literal(s.to_string()),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { components })
    }
}

impl fmt::Display for PathGlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, component) in self.components.iter().enumerate() {
            if i != 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", component.as_ref())?;
        }
        Ok(())
    }
}

impl PathGlob {
    pub fn matches_components<'a>(&self, target: impl IntoIterator<Item = &'a str>) -> bool {
        let mut components = target
            .into_iter()
            .map(|x| x.trim())
            .filter(|x| !x.is_empty());
        let mut glob_i = 0;

        loop {
            let glob = if let Some(glob) = self.components.get(glob_i) {
                glob
            } else {
                if components.next().is_some() {
                    return false;
                }
                break;
            };

            if matches!(glob, GlobComponent::AnyMany) {
                glob_i += 1;
                if let Some(next_glob) = self.components.get(glob_i) {
                    loop {
                        if let Some(next) = components.next() {
                            if next_glob.matches(next) {
                                glob_i += 1;
                                break;
                            }
                        } else {
                            return false;
                        }
                    }
                } else {
                    break;
                }
                continue;
            }
            glob_i += 1;
            let component = match components.next() {
                Some(x) => x,
                None => return false,
            };
            if !glob.matches(component) {
                return false;
            }
        }

        true
    }

    pub fn matches(&self, target: &str) -> bool {
        self.matches_components(target.split('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_globs() {
        let glob: PathGlob = "xyz".parse().unwrap();
        assert!(glob.matches("xyz"));
        assert!(!glob.matches("xyzx"));
        assert!(!glob.matches("xy"));
        assert!(!glob.matches("xyz/cdf"));
        assert!(!glob.matches("cdf/xyz"));

        let glob: PathGlob = "*xyz".parse().unwrap();
        assert!(glob.matches("xyz"));
        assert!(glob.matches("sdsxyz"));
        assert!(glob.matches("sxyz"));
        assert!(!glob.matches("xyzds"));

        let glob: PathGlob = "xyz*".parse().unwrap();
        assert!(glob.matches("xyz"));
        assert!(glob.matches("xyzsds"));
        assert!(glob.matches("xyzs"));
        assert!(!glob.matches("dsxyz"));

        let glob: PathGlob = "*xyz*".parse().unwrap();
        assert!(glob.matches("xyz"));
        assert!(glob.matches("sdsxyzsds"));
        assert!(glob.matches("sxyzs"));
        assert!(!glob.matches("dsxy zds"));

        let glob: PathGlob = "t/*/y".parse().unwrap();
        assert!(glob.matches("t/x/y"));
        assert!(glob.matches("t/sdfsdf/y"));
        assert!(!glob.matches("t//y"));
        assert!(!glob.matches("t/d/x/y"));
        assert!(!glob.matches("t/y"));

        let glob: PathGlob = "t/**/y".parse().unwrap();
        assert!(glob.matches("t/x/y"));
        assert!(glob.matches("t/sdfsdf/y"));
        assert!(glob.matches("t//y"));
        assert!(glob.matches("t/d/x/y"));
        assert!(glob.matches("t/y"));
        assert!(!glob.matches("t/y/d"));
        assert!(!glob.matches("d/t/y"));

        let glob: PathGlob = "t/**".parse().unwrap();
        assert!(glob.matches("t"));
        assert!(glob.matches("t/x/y"));
        assert!(glob.matches("t/sdfsdf/y"));
        assert!(glob.matches("t//y"));
        assert!(glob.matches("t/d/x/y"));
        assert!(glob.matches("t/y"));
        assert!(glob.matches("t/"));
        assert!(glob.matches("t/d/d/d/d/d"));
        assert!(!glob.matches("d/"));
        assert!(!glob.matches("d/t/d"));

        let glob: PathGlob = "t/#[0-9]+/**".parse().unwrap();
        assert!(glob.matches("t/30/product"));
        assert!(glob.matches("t/30"));
        assert!(glob.matches("t/1"));
        assert!(glob.matches("t/999999999"));
        assert!(!glob.matches("t/"));
        assert!(!glob.matches("t/x"));
        assert!(!glob.matches("t/x/x"));
    }

    #[test]
    fn test_glob_order() {
        let glob1: PathGlob = "xyz".parse().unwrap();
        let glob2: PathGlob = "xyz".parse().unwrap();
        assert_eq!(glob1.partial_cmp(&glob2), Some(Ordering::Equal));
        let glob1: PathGlob = "xyz/xyz".parse().unwrap();
        let glob2: PathGlob = "xyz".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/xyz".parse().unwrap();
        let glob2: PathGlob = "xyz/*".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/xyz".parse().unwrap();
        let glob2: PathGlob = "xyz/**".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/xyz".parse().unwrap();
        let glob2: PathGlob = "xyz/*test".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/xyz/xyz".parse().unwrap();
        let glob2: PathGlob = "xyz/*test".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/*test".parse().unwrap();
        let glob2: PathGlob = "xyz/*".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/*test".parse().unwrap();
        let glob2: PathGlob = "xyz/**".parse().unwrap();
        assert!(glob1 < glob2);
        let glob1: PathGlob = "xyz/*".parse().unwrap();
        let glob2: PathGlob = "xyz/**".parse().unwrap();
        assert!(glob1 < glob2);
    }
}
