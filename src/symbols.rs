use errors::*;
use goblin::elf::Elf;
use goblin::mach::{self, Mach, MachO};
use goblin::Object;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct ExtractedSymbol {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedSymbols {
    pub symbols: Vec<ExtractedSymbol>,
}

impl From<Vec<ExtractedSymbol>> for ExtractedSymbols {
    fn from(symbols: Vec<ExtractedSymbol>) -> Self {
        ExtractedSymbols { symbols }
    }
}

fn parse_elf(elf: &Elf) -> Result<ExtractedSymbols, WError> {
    let mut symbols = vec![];

    for symbol in elf.dynsyms
        .iter()
        .filter(|symbol| symbol.st_info == 0x12 || symbol.st_info == 0x22)
    {
        let name = elf.dynstrtab
            .get(symbol.st_name)
            .ok_or(WError::ParseError)?
            .map_err(|_| WError::ParseError)?
            .to_string();
        let extracted_symbol = ExtractedSymbol { name };
        symbols.push(extracted_symbol);
    }
    Ok(symbols.into())
}

// Mach-O symbols don't include any sizes, so we need to extract all the symbols
// from the text section, and for each symbol, find the one with the smallest
// offset immediately after the reference symbol, in order to guess the reference
// symbol's size (alignment included).
fn parse_macho(macho: &MachO) -> Result<ExtractedSymbols, WError> {
    let mut symbols = vec![];

    // Start by finding the boundaries of the text section
    let mut text_offset = None;
    let mut text_size = None;
    for section in macho.segments.sections() {
        for segment in section {
            if let Ok((
                mach::segment::Section {
                    sectname: [b'_', b'_', b't', b'e', b'x', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    segname: [b'_', b'_', b'T', b'E', b'X', b'T', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    size,
                    offset,
                    ..
                },
                _,
            )) = segment
            {
                text_offset = Some(offset as usize);
                text_size = Some(size as usize);
            }
        }
    }
    let text_offset = text_offset.ok_or(WError::ParseError)?;
    let text_size = text_size.ok_or(WError::ParseError)?;

    // Extract the symbols we are interested in
    for symbol in macho.symbols.as_ref().ok_or(WError::ParseError)?.iter() {
        match symbol {
            Ok((
                name,
                mach::symbols::Nlist {
                    n_type: 0xf,
                    n_sect: 1,
                    n_value,
                    ..
                },
            )) if name.len() > 1 && name.starts_with('_') =>
            {
                let extracted_symbol = ExtractedSymbol {
                    name: name[1..].to_string(),
                };
                let offset = n_value as usize;
                if offset < text_offset || offset >= text_offset + text_size {
                    continue;
                }
                symbols.push(extracted_symbol);
            }
            _ => {}
        }
    }
    Ok(symbols.into())
}

pub fn exported_symbols<P: AsRef<Path>>(path: P) -> Result<ExtractedSymbols, WError> {
    let mut buffer = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;
    let symbols = match Object::parse(&buffer).map_err(|_| WError::ParseError)? {
        Object::Mach(Mach::Binary(macho)) => parse_macho(&macho),
        Object::Elf(elf) => parse_elf(&elf),
        _ => xbail!(WError::Unsupported),
    }?;
    Ok(symbols)
}
