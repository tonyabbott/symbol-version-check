use crate::symbols::SymbolVersion;
use crate::version::NamespacedVersion;
use anyhow::anyhow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VersionRequirements {
    requirements: HashMap<String, NamespacedVersion>,
}

impl VersionRequirements {
    pub fn parse(requirements: &[String]) -> anyhow::Result<VersionRequirements> {
        let requirements = requirements.iter().try_fold(HashMap::new(), |mut acc, v| {
            let nv = NamespacedVersion::parse(v)?;
            if acc.contains_key(&nv.namespace) {
                return Err(anyhow!("Duplicate namespace: {}", nv.namespace));
            }
            acc.insert(nv.namespace.clone(), nv);
            Ok(acc)
        })?;
        Ok(VersionRequirements { requirements })
    }

    pub fn check_symbols(&self, symbols: &[SymbolVersion]) -> Vec<SymbolVersion> {
        symbols
            .iter()
            .filter(|symbol| {
                self.requirements
                    .get(&symbol.version.namespace)
                    .is_some_and(|req| symbol.version > *req)
            })
            .cloned()
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_requirements_single_version() {
        let requirements = vec!["GLIBC_2.17".to_string()];
        let result = VersionRequirements::parse(&requirements).unwrap();
        assert_eq!(result.requirements.len(), 1);
        assert!(result.requirements.contains_key("GLIBC"));
        assert_eq!(
            result.requirements["GLIBC"],
            NamespacedVersion::parse("GLIBC_2.17").unwrap()
        );
    }

    #[test]
    fn parse_requirements_multiple_namespaces() {
        let requirements = vec!["GLIBC_2.17".to_string(), "GLIBCXX_3.4.21".to_string()];
        let result = VersionRequirements::parse(&requirements).unwrap();
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(
            result.requirements["GLIBC"],
            NamespacedVersion::parse("GLIBC_2.17").unwrap()
        );
        assert_eq!(
            result.requirements["GLIBCXX"],
            NamespacedVersion::parse("GLIBCXX_3.4.21").unwrap()
        );
    }

    #[test]
    fn parse_requirements_duplicate_namespace_fails() {
        let requirements = vec!["GLIBC_2.17".to_string(), "GLIBC_2.18".to_string()];
        let result = VersionRequirements::parse(&requirements);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate namespace")
        );
    }

    #[test]
    fn parse_requirements_invalid_version_format() {
        let requirements = vec!["invalid".to_string()];
        let result = VersionRequirements::parse(&requirements);
        assert!(result.is_err());
    }

    #[test]
    fn parse_requirements_empty_list() {
        let requirements = vec![];
        let result = VersionRequirements::parse(&requirements).unwrap();
        assert!(result.requirements.is_empty());
    }
}
