use std::fmt;

use axoupdater::Version;

/// A release's git tag, e.g. `v0.4.0`, as GitHub names it in the address of
/// the release's page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTagDTO(pub String);

impl ReleaseTagDTO {
    /// The version the tag is for; none when the tag is not `v` followed by
    /// a version.
    pub fn version(&self) -> Option<Version> {
        self.0.strip_prefix('v')?.parse().ok()
    }
}

impl From<String> for ReleaseTagDTO {
    fn from(tag: String) -> Self {
        Self(tag)
    }
}

impl fmt::Display for ReleaseTagDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(text: &str) -> ReleaseTagDTO {
        ReleaseTagDTO::from(text.to_owned())
    }

    #[test]
    fn reads_the_version_from_the_tag() {
        assert_eq!(tag("v0.4.0").version(), Some("0.4.0".parse().unwrap()));
        assert_eq!(
            tag("v1.0.0-rc.1").version(),
            Some("1.0.0-rc.1".parse().unwrap())
        );
    }

    #[test]
    fn a_tag_without_a_version_has_none() {
        assert_eq!(tag("0.4.0").version(), None);
        assert_eq!(tag("vnext").version(), None);
        assert_eq!(tag("").version(), None);
    }
}
