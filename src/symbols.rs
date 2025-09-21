use crate::version::NamespacedVersion;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolVersion {
    pub name: String,
    pub version: NamespacedVersion,
    pub file: Option<String>,
}

impl SymbolVersion {
    #[cfg(test)]
    pub fn parse(name: &str, version: &str, file: Option<String>) -> anyhow::Result<SymbolVersion> {
        let version = NamespacedVersion::parse(&version)?;
        Ok(SymbolVersion {
            name: name.to_string(),
            version,
            file,
        })
    }

    pub fn try_demangle_cpp_name(&self) -> Option<String> {
        cpp_demangle::Symbol::new(&self.name)
            .map(|symbol| symbol.to_string())
            .ok()
    }

    pub fn try_demangle_rust_name(&self) -> Option<String> {
        rustc_demangle::try_demangle(&self.name)
            .map(|demangled| demangled.to_string())
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demangles_cpp_name() {
        let symbol = SymbolVersion::parse(
            "_ZNKSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE12find_last_ofEPKcmm",
            "LIB_1",
            None,
        )
        .unwrap();
        assert_eq!(
            symbol.try_demangle_cpp_name(),
            Some("std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> >::find_last_of(char const*, unsigned long, unsigned long) const".to_string())
        );
    }

    #[test]
    fn doesnt_demangle_unmangled_cpp_name() {
        let symbol = SymbolVersion::parse("main", "LIB_1", None).unwrap();
        assert_eq!(symbol.try_demangle_cpp_name(), None);
    }

    #[test]
    fn demangles_rust_name() {
        let symbol = SymbolVersion::parse(
            "_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hf851721abb1b401eE",
            "LIB_1",
            None,
        )
        .unwrap();
        assert_eq!(
            symbol.try_demangle_rust_name(),
            Some("std::rt::lang_start::{{closure}}::hf851721abb1b401e".to_string())
        );
    }

    #[test]
    fn doesnt_demangle_unmangled_rust_name() {
        let symbol = SymbolVersion::parse("main", "LIB_1", None).unwrap();
        assert_eq!(symbol.try_demangle_rust_name(), None);
    }
}
