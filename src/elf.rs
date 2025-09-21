use crate::symbols::SymbolVersion;
use crate::version::NamespacedVersion;
use anyhow::{Context, anyhow};
use object::read::elf::{ElfFile, ElfFile32, ElfFile64, FileHeader};
use object::{Endianness, FileKind, Object, ObjectSymbol};
use std::fs;
use std::path::Path;

pub fn get_dyn_undef_symbols(file_to_check: &Path) -> anyhow::Result<Vec<SymbolVersion>> {
    let data = fs::read(file_to_check).with_context(|| "Error reading file")?;
    match FileKind::parse(&*data).with_context(|| "Error parsing file")? {
        FileKind::Elf32 => get_elf_dyn_undef_symbols(ElfFile32::parse(&*data)?, &data),
        FileKind::Elf64 => get_elf_dyn_undef_symbols(ElfFile64::parse(&*data)?, &data),
        _ => Err(anyhow!("Unsupported file format")),
    }
}

fn get_elf_dyn_undef_symbols<'data, Elf: FileHeader<Endian = Endianness>>(
    elf: ElfFile<'data, Elf>,
    data: &'data [u8],
) -> anyhow::Result<Vec<SymbolVersion>> {
    let version_table = elf
        .elf_header()
        .sections(elf.endianness(), data)
        .with_context(|| "Error getting section table")?
        .versions(elf.endianness(), data)
        .with_context(|| "Error getting version table")?
        .ok_or_else(|| anyhow!("No version table found"))?;

    elf.dynamic_symbols()
        .filter(|s| s.is_undefined())
        .map(|symbol| {
            let name = symbol.name().with_context(|| "Error reading symbol name")?;
            let version_index = version_table.version_index(elf.endianness(), symbol.index());
            let version = version_table
                .version(version_index)
                .with_context(|| "Error reading symbol version info")?;
            match version {
                Some(v) => {
                    let version = String::from_utf8_lossy(v.name()).to_string();
                    let file = v.file().map(|f| String::from_utf8_lossy(f).to_string());
                    match NamespacedVersion::parse(&version) {
                        Ok(version) => Ok(Some(SymbolVersion {
                            name: name.to_string(),
                            version,
                            file,
                        })),
                        Err(_) => Ok(None),
                    }
                }
                None => Ok(None),
            }
        })
        .filter_map(|symbol| symbol.transpose())
        .collect::<Result<Vec<_>, _>>()
}
