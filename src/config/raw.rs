use std::{
    collections::BTreeMap,
    fmt,
    fs,
    str::{self, FromStr},
};

use anyhow::{bail, Context as _, Result};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use url::Url;

use crate::{
    context::Context,
    regex,
    util::{load_toml, not_found_err, remove_file_if_exists},
};

use super::{Config, GitReference, Package, Repository, Source};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RawConfig {
    #[serde(default)]
    pub pkgs: BTreeMap<String, RawPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct RawPackage {
    #[serde(skip)]
    pub name:    String,
    #[serde(default = "BoolExpr::default")]
    pub enabled: BoolExpr,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bins: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub desc: String,

    /// A GitHub source.
    pub github:    Option<GitHubSource>,
    /// A Git source.
    pub git:       Option<Url>,
    /// A Gist source.
    pub gist:      Option<GistSource>,
    /// A downloadable file.
    pub remote:    Option<Url>,
    /// The Git reference to checkout.
    #[serde(flatten)]
    pub reference: Option<GitReference>,
}

/// A GitHub source identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubSource {
    /// The GitHub username / organization.
    pub owner:  String,
    /// The GitHub repository name.
    pub repo:   String,
    /// The Source of package.
    pub source: GithubSourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum GithubSourceType {
    Git,
    Release,
}

/// A Gist source identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GistSource {
    /// The GitHub owner.
    pub owner:      Option<String>,
    /// The Gist identifier.
    pub identifier: String,
}

impl fmt::Display for GistSource {
    /// Displays as "[{owner}/]{identifier}".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self { owner: Some(owner), identifier } => write!(f, "{owner}/{identifier}"),
            Self { owner: None, identifier } => write!(f, "{identifier}"),
        }
    }
}

impl fmt::Display for GitHubSource {
    /// Displays as "{owner}/{repo}:{source_type}".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}:{}", self.owner, self.repo, self.source)
    }
}

macro_rules! impl_serialize_as_str {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
    };
}

impl_serialize_as_str! { GistSource }
impl_serialize_as_str! { GitHubSource }

/// Produced when we fail to parse a Gist source.
#[derive(Debug, Error)]
#[error("invalid Gist source `{}`, should be in format of OWNER/HASH", self.0)]
pub struct ParseGistSourceError(String);

impl FromStr for GistSource {
    type Err = ParseGistSourceError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = regex!("^((?P<owner>[a-zA-Z0-9_-]+)/)?(?P<identifier>[a-fA-F0-9]+)$");
        let captures = re
            .captures(s)
            .ok_or_else(|| ParseGistSourceError(s.to_string()))?;
        let owner = captures.name("owner").map(|m| m.as_str().to_string());
        let identifier = captures.name("identifier").unwrap().as_str().to_string();
        Ok(Self { owner, identifier })
    }
}

/// Produced when we fail to parse a GitHub source.
#[derive(Debug, Error)]
#[error("invalid GitHub source `{}`, should be in format of OWNER/REPO:<release|git>", self.0)]
pub struct ParseGitHubSourceError(String);

impl FromStr for GitHubSource {
    type Err = ParseGitHubSourceError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = regex!(
            "^(?P<owner>[a-zA-Z0-9_-]+)/(?P<name>[a-zA-Z0-9\\._-]+)(:(?P<source>(git|release)))?$"
        );
        let captures = re
            .captures(s)
            .ok_or_else(|| ParseGitHubSourceError(s.to_string()))?;
        let owner = captures.name("owner").unwrap().as_str().to_string();
        let name = captures.name("name").unwrap().as_str().to_string();
        let source = captures
            .name("source")
            .map(|m| match m.as_str() {
                "git" => GithubSourceType::Git,
                "release" => GithubSourceType::Release,
                _ => unreachable!(),
            })
            .unwrap_or(GithubSourceType::Release);
        Ok(Self { owner, repo: name, source })
    }
}

macro_rules! impl_deserialize_from_str {
    ($module:ident, $name:ident, $expecting:expr) => {
        mod $module {
            use super::*;

            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = $name;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str($expecting)
                }

                fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    $name::from_str(value).map_err(|e| de::Error::custom(e.to_string()))
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_str(Visitor)
                }
            }
        }
    };
}

impl_deserialize_from_str! { gist_source, GistSource, "a Gist source" }
impl_deserialize_from_str! { github_source, GitHubSource, "a GitHub source" }

/// Deserialize the remaining keys. Empty tables are coerced to [`None`].
fn deserialize_rest_toml_value<'de, D>(deserializer: D) -> Result<Option<toml::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: toml::Value = de::Deserialize::deserialize(deserializer)?;
    Ok(match value {
        toml::Value::Table(table) if table.is_empty() => None,
        value => Some(value),
    })
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum BoolExpr {
    Bool(bool),
    Command(String),
}

impl BoolExpr {
    pub fn eval(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Command(cmd) => std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .is_ok_and(|o| o.status.success()),
        }
    }
}

impl Default for BoolExpr {
    fn default() -> Self {
        Self::Bool(true)
    }
}

impl From<bool> for BoolExpr {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl RawConfig {
    /// Load the configuration from the given path.
    pub fn load(ctx: &Context) -> Result<Self> {
        let mut cfg = match load_toml(&ctx.config_file) {
            Err(e) if not_found_err(e.root_cause()) => RawConfig::init(ctx)?,
            cfg => cfg.with_context(|| format!("failed to load {}", ctx.config_file.display()))?,
        };

        cfg.pkgs
            .iter_mut()
            .for_each(|(name, p)| p.name = name.clone());

        Ok(cfg)
    }

    fn init(ctx: &Context) -> Result<Self> {
        remove_file_if_exists(&ctx.lock_file)
            .with_context(|| format!("failed to remove lock file {}", ctx.lock_file.display()))?;

        let default = include_str!("packages.toml");
        fs::write(&ctx.config_file, default)
            .with_context(|| format!("failed to init {}", ctx.config_file.display()))?;
        Ok(toml::from_str(default)?)
    }
}

impl TryInto<Config> for RawConfig {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Config> {
        self.pkgs
            .into_iter()
            .map(|(k, v)| Ok((k, v.try_into()?)))
            .collect::<Result<_>>()
            .map(|pkgs| Config { pkgs })
    }
}

impl TryInto<Package> for RawPackage {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Package> {
        let source = match (self.github, self.git, self.gist, self.remote) {
            (Some(github), None, None, None) => match github.source {
                GithubSourceType::Git => {
                    let url = format!("https://github.com/{}/{}", github.owner, github.repo);
                    let url = Url::parse(&url).with_context(|| {
                        format!("failed to construct GitHub URL using `{}`", github)
                    })?;

                    Source::Git { url, reference: self.reference }
                }
                GithubSourceType::Release => {
                    let tag = match self.reference {
                        None => None,
                        Some(GitReference::Tag(tag)) => Some(tag),
                        _ => bail!("expected a tag reference, found `{:?}`", self.reference),
                    };
                    Source::Release {
                        repo: Repository { owner: github.owner, name: github.repo },
                        tag,
                    }
                }
            },

            (None, Some(git), None, None) =>
                Source::Git { url: git, reference: self.reference },
            (None, None, Some(gist), None) => {
                let url = format!("https://gist.github.com/{gist}");
                let url = Url::parse(&url)
                    .with_context(|| format!("failed to construct Gist URL using `{gist}`"))?;

                Source::Git { url, reference: self.reference }
            }
            (None, None, None, Some(remote)) => Source::Remote { url: remote },
            (None, None, None, None) => bail!("no source field found in package `{}`", self.name),
            _ => bail!("multiple source fields found in package `{}`", self.name),
        };

        let bins = if self.bins.is_empty() {
            vec![self.name.clone()]
        } else {
            self.bins
        };

        Ok(Package {
            name: self.name.clone(),
            enabled: self.enabled.eval(),
            bins,
            source,
            desc: self.desc,
        })
    }
}

impl Into<RawPackage> for Package {
    fn into(self) -> RawPackage {
        let mut pkg = RawPackage::default();

        match self.source {
            Source::Git { url, reference } => {
                pkg.git = Some(url);
                pkg.reference = reference;
            }
            Source::Remote { url } => pkg.remote = Some(url),
            Source::Release { repo, tag } => {
                pkg.github = Some(GitHubSource {
                    owner:  repo.owner,
                    repo:   repo.name,
                    source: GithubSourceType::Release,
                });
                pkg.reference = tag.map(GitReference::Tag);
            }
        }

        pkg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn gist_source_to_string() {
        let test = GistSource {
            owner:      None,
            identifier: "794760e9a2d066559dae252b0a3a1086".to_string(),
        };
        assert_eq!(test.to_string(), "794760e9a2d066559dae252b0a3a1086");
    }

    #[test]
    fn gist_source_to_string_with_owner() {
        let test = GistSource {
            owner:      Some("wfxr".to_string()),
            identifier: "794760e9a2d066559dae252b0a3a1086".to_string(),
        };
        assert_eq!(test.to_string(), "wfxr/794760e9a2d066559dae252b0a3a1086");
    }

    #[test]
    fn github_source_to_string() {
        let test = GitHubSource {
            owner:  "wfxr".to_string(),
            repo:   "rpk".to_string(),
            source: GithubSourceType::Release,
        };
        assert_eq!(test.to_string(), "wfxr/rpk:release");
    }

    #[test]
    fn github_source_with_git_source_to_string() {
        let test = GitHubSource {
            owner:  "wfxr".to_string(),
            repo:   "rpk".to_string(),
            source: GithubSourceType::Git,
        };
        assert_eq!(test.to_string(), "wfxr/rpk:git");
    }

    #[derive(Debug, Deserialize)]
    struct MockGistSource {
        g: GistSource,
    }

    #[test]
    fn gist_source_deserialize() {
        let MockGistSource { g } =
            toml::from_str("g = 'wfxr/794760e9a2d066559dae252b0a3a1086'").unwrap();
        assert_eq!(g, GistSource {
            owner:      Some("wfxr".to_string()),
            identifier: "794760e9a2d066559dae252b0a3a1086".to_string(),
        });
    }

    #[test]
    fn gist_source_deserialize_two_slashes() {
        let error =
            toml::from_str::<MockGistSource>("g = 'wfxr/794760e9a2d066559dae252b0a3a1086/test'")
                .unwrap_err();
        assert_eq!(
            error.to_string(),
            "TOML parse error at line 1, column 5
  |
1 | g = 'wfxr/794760e9a2d066559dae252b0a3a1086/test'
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
invalid Gist source `wfxr/794760e9a2d066559dae252b0a3a1086/test`, should be in format of OWNER/HASH
"
        );
    }

    #[test]
    fn gist_source_deserialize_not_hex() {
        let error = toml::from_str::<MockGistSource>("g = 'nothex'").unwrap_err();
        assert_eq!(
            error.to_string(),
            "TOML parse error at line 1, column 5
  |
1 | g = 'nothex'
  |     ^^^^^^^^
invalid Gist source `nothex`, should be in format of OWNER/HASH
"
        );
    }

    #[derive(Debug, Deserialize)]
    struct MockGitHubSource {
        g: GitHubSource,
    }

    #[test]
    fn github_source_deserialize() {
        let MockGitHubSource { g } = toml::from_str("g = 'wfxr/rpk'").unwrap();
        assert_eq!(g, GitHubSource {
            owner:  "wfxr".to_string(),
            repo:   "rpk".to_string(),
            source: GithubSourceType::Release,
        });

        let MockGitHubSource { g } = toml::from_str("g = 'wfxr/rpk:release'").unwrap();
        assert_eq!(g, GitHubSource {
            owner:  "wfxr".to_string(),
            repo:   "rpk".to_string(),
            source: GithubSourceType::Release,
        });
    }

    #[test]
    fn github_source_with_git_source_deserialize() {
        let MockGitHubSource { g } = toml::from_str("g = 'wfxr/rpk:git'").unwrap();
        assert_eq!(g, GitHubSource {
            owner:  "wfxr".to_string(),
            repo:   "rpk".to_string(),
            source: GithubSourceType::Git,
        });
    }

    #[test]
    fn github_source_deserialize_two_slashes() {
        let error = toml::from_str::<MockGitHubSource>("g = 'wfxr/rpk/test'").unwrap_err();
        assert_eq!(
            error.to_string(),
            "TOML parse error at line 1, column 5
  |
1 | g = 'wfxr/rpk/test'
  |     ^^^^^^^^^^^^^^^
invalid GitHub source `wfxr/rpk/test`, should be in format of OWNER/REPO:<release|git>
"
        );
    }

    #[test]
    fn github_source_deserialize_no_slashes() {
        let error = toml::from_str::<MockGitHubSource>("g = 'noslash'").unwrap_err();
        assert_eq!(
            error.to_string(),
            "TOML parse error at line 1, column 5
  |
1 | g = 'noslash'
  |     ^^^^^^^^^
invalid GitHub source `noslash`, should be in format of OWNER/REPO:<release|git>
"
        );
    }

    #[test]
    fn raw_plugin_deserialize_git() {
        let expected = RawPackage {
            git: Some(Url::parse("https://github.com/wfxr/rpk").unwrap()),
            ..Default::default()
        };
        let acutal: RawPackage = toml::from_str("git = 'https://github.com/wfxr/rpk'").unwrap();
        assert_eq!(acutal, expected);
    }

    #[test]
    fn raw_plugin_deserialize_github_release() {
        let expected = RawPackage {
            github: Some(GitHubSource {
                owner:  "wfxr".into(),
                repo:   "rpk".into(),
                source: GithubSourceType::Release,
            }),
            ..Default::default()
        };
        let plugin: RawPackage = toml::from_str("github = 'wfxr/rpk:release'").unwrap();
        assert_eq!(plugin, expected);
    }

    #[test]
    fn raw_plugin_deserialize_github_git() {
        let expected = RawPackage {
            github: Some(GitHubSource {
                owner:  "wfxr".into(),
                repo:   "rpk".into(),
                source: GithubSourceType::Git,
            }),
            ..Default::default()
        };
        let plugin: RawPackage = toml::from_str("github = 'wfxr/rpk:git'").unwrap();
        assert_eq!(plugin, expected);
    }
}
