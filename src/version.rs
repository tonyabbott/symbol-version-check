use anyhow::anyhow;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamespacedVersion {
    pub namespace: String,
    pub version: Version,
}

impl NamespacedVersion {
    pub fn parse(requirement: &str) -> anyhow::Result<NamespacedVersion> {
        match requirement.rfind(['_']) {
            Some(split_pos) => {
                let namespace = requirement[..split_pos].to_string();
                if namespace.is_empty() {
                    return Err(anyhow!(
                        "Missing namespace in namespaced version: {}",
                        requirement
                    ));
                }
                let version = &requirement[split_pos + 1..];
                let starts_with_digit = version
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false);
                if starts_with_digit {
                    let max_version = Version::parse(version)?;
                    Ok(NamespacedVersion {
                        namespace,
                        version: max_version,
                    })
                } else {
                    Err(anyhow!("Invalid namespaced version: {}", requirement))
                }
            }
            None => Err(anyhow!("Invalid namespaced version: {}", requirement)),
        }
    }
}

impl Display for NamespacedVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.namespace, self.version)
    }
}

/// Version number. A version number consists of a series of one or more non-negative integers separated by periods.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    values: Vec<u32>,
}

impl Version {
    pub fn parse(version: &str) -> anyhow::Result<Version> {
        let mut values: Vec<u32> = version
            .split('.')
            .map(|s| s.parse::<u32>())
            .collect::<Result<_, _>>()
            .map_err(|e| anyhow!("Invalid version {}: {}", version, e))?;
        // Remove trailing zeroes from the version number, for our purposes they are not significant
        match values.iter().rposition(|&x| x != 0) {
            Some(last_non_zero) => values.truncate(last_non_zero + 1),
            None => values.clear(),
        }
        Ok(Version { values })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.values
                .iter()
                .map(|&v| v.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Version({})", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_parse_empty_is_error() {
        assert!(Version::parse("").is_err());
    }

    #[test]
    fn version_parse_non_numeric_is_error() {
        assert!(Version::parse("x").is_err());
    }

    #[test]
    fn version_parse_negative_is_error() {
        assert!(Version::parse("-1").is_err());
    }

    #[test]
    fn version_parse_trailing_dot_is_error() {
        assert!(Version::parse("1.").is_err());
    }

    #[test]
    fn version_parse_leading_dot_is_error() {
        assert!(Version::parse(".1").is_err());
    }

    #[test]
    fn version_parse_single_zero_is_empty() {
        assert!(Version::parse("0").unwrap().values.is_empty());
    }

    #[test]
    fn version_parse_single() {
        assert_eq!(Version::parse("1").unwrap().values, vec![1]);
    }

    #[test]
    fn version_parse_triplet() {
        assert_eq!(Version::parse("1.2.3").unwrap().values, vec![1, 2, 3]);
    }

    #[test]
    fn version_parse_trailing_zeroes_stripped() {
        assert_eq!(Version::parse("0.1.0.0").unwrap().values, vec![0, 1]);
    }

    #[test]
    fn version_cmp() {
        assert!(Version::parse("1").unwrap() < Version::parse("42").unwrap());
        assert!(Version::parse("1.2.3").unwrap() < Version::parse("1.2.4").unwrap());
        assert!(Version::parse("1.2").unwrap() < Version::parse("1.2.1").unwrap());
        assert!(Version::parse("1.2.3").unwrap() > Version::parse("1.2").unwrap());
    }

    #[test]
    fn version_equality() {
        assert_eq!(Version::parse("1").unwrap(), Version::parse("1").unwrap());
        assert_eq!(
            Version::parse("1.42").unwrap(),
            Version::parse("1.42").unwrap()
        );
        assert_eq!(
            Version::parse("1.2").unwrap(),
            Version::parse("1.2.0").unwrap()
        );
    }

    #[test]
    fn namespaced_version_parses() {
        let version = NamespacedVersion::parse("GLIBC_2.17").unwrap();
        assert_eq!(version.namespace, "GLIBC");
        assert_eq!(version.version, Version::parse("2.17").unwrap());
    }

    #[test]
    fn namespaced_version_parses_with_compound_namespace() {
        let version = NamespacedVersion::parse("GLIB_C_2.17").unwrap();
        assert_eq!(version.namespace, "GLIB_C");
        assert_eq!(version.version, Version::parse("2.17").unwrap());
    }

    #[test]
    fn namespaced_version_parse_error_when_no_namespace() {
        let result = NamespacedVersion::parse("2.17");
        assert!(result.is_err());
    }

    #[test]
    fn namespaced_version_parse_error_when_empty_namespace() {
        let result = NamespacedVersion::parse("_2.17");
        assert!(result.is_err());
    }

    #[test]
    fn namespaced_version_parse_error_when_no_version() {
        let result = NamespacedVersion::parse("GLIBC_");
        assert!(result.is_err());
    }

    #[test]
    fn namespaced_version_parse_error_when_no_separator() {
        let result = NamespacedVersion::parse("GLIBC");
        assert!(result.is_err());
    }

    #[test]
    fn namespaced_version_equal_when_namespace_and_version_equal() {
        let version1 = NamespacedVersion::parse("GLIBC_2.17").unwrap();
        let version2 = NamespacedVersion::parse("GLIBC_2.17").unwrap();
        assert_eq!(version1, version2);
    }

    #[test]
    fn namespaced_version_not_equal_when_namespace_not_equal() {
        let version1 = NamespacedVersion::parse("GLIBC_2.17").unwrap();
        let version2 = NamespacedVersion::parse("GLIBX_2.17").unwrap();
        assert_ne!(version1, version2);
    }

    #[test]
    fn namespaced_version_not_equal_when_versions_not_equal() {
        let version1 = NamespacedVersion::parse("GLIBC_2.17").unwrap();
        let version2 = NamespacedVersion::parse("GLIBC_2.18").unwrap();
        assert_ne!(version1, version2);
    }

    #[test]
    fn namespaced_version_cmp() {
        assert!(
            NamespacedVersion::parse("X_1").unwrap() < NamespacedVersion::parse("X_42").unwrap()
        );
        assert!(
            NamespacedVersion::parse("X_1.2.3").unwrap()
                < NamespacedVersion::parse("X_1.2.4").unwrap()
        );
        assert!(
            NamespacedVersion::parse("X_1.2").unwrap()
                < NamespacedVersion::parse("X_1.2.1").unwrap()
        );
        assert!(
            NamespacedVersion::parse("X_1.2.3").unwrap()
                > NamespacedVersion::parse("X_1.2").unwrap()
        );
    }

    #[test]
    fn namespaced_version_cmp_with_different_namespaces() {
        assert!(
            NamespacedVersion::parse("X_1").unwrap() < NamespacedVersion::parse("Y_1").unwrap()
        );
    }
}
