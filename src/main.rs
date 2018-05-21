extern crate clap;
#[macro_use]
extern crate failure;
extern crate goblin;
extern crate parity_wasm;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate xfailure;

mod config;
mod errors;
mod functions_ids;
mod functions_names;
mod map;
mod sections;
mod symbols;

use config::*;
use errors::*;
use functions_ids::*;
use functions_names::*;
use map::*;
use parity_wasm::elements::{
    self, External, ImportEntry, ImportSection, Internal, Module, NameSection, Section,
};
use sections::*;
use std::path::Path;

pub const BUILTIN_PREFIX: &str = "builtin_";

#[derive(Debug)]
pub struct Builtin {
    pub name: String,
    pub original_function_id: Option<u32>,
    pub function_type_id: Option<u32>,
}

impl Builtin {
    pub fn new(name: String) -> Self {
        Builtin {
            name,
            original_function_id: None,
            function_type_id: None,
        }
    }

    pub fn import_name(&self) -> String {
        format!("{}{}", BUILTIN_PREFIX, self.name)
    }
}

fn add_function_type_id_to_builtins(
    module: &Module,
    builtins: &mut Vec<Builtin>,
) -> Result<(), WError> {
    let functions_type_ids = module.function_section().unwrap().entries();
    for builtin in builtins.iter_mut() {
        let function_type_id =
            functions_type_ids[builtin.original_function_id.unwrap() as usize].type_ref();
        builtin.function_type_id = Some(function_type_id);
    }
    Ok(())
}

fn retain_only_used_builtins(module: &Module, builtins: &mut Vec<Builtin>) -> Result<(), WError> {
    let export_section = module.export_section().expect("No export section");

    for entry in export_section.entries() {
        let internal = entry.internal();
        let function_id = match internal {
            Internal::Function(function_id) => *function_id,
            _ => continue,
        };
        let field = entry.field();
        for builtin in builtins.iter_mut() {
            if field == builtin.name {
                assert!(builtin.original_function_id.is_none());
                builtin.original_function_id = Some(function_id);
                break;
            }
        }
    }

    builtins.retain(|builtin| builtin.original_function_id.is_some());
    Ok(())
}

fn add_import_section_if_missing(module: &mut Module) -> Result<(), WError> {
    if module.import_section().is_some() {
        return Ok(());
    }
    let import_section = ImportSection::with_entries(vec![]);
    let import_section_idx = find_type_section_idx(&module).unwrap() + 1;
    module
        .sections_mut()
        .insert(import_section_idx, Section::Import(import_section));
    Ok(())
}

fn prepend_builtin_to_import_section(module: &mut Module, builtin: &Builtin) -> Result<(), WError> {
    let import_name = builtin.import_name();
    let external = External::Function(builtin.function_type_id.unwrap());
    let import_entry = ImportEntry::new("env".to_string(), import_name, external);
    module
        .import_section_mut()
        .unwrap()
        .entries_mut()
        .insert(0, import_entry);
    Ok(())
}

fn prepend_builtin_to_names_section(module: &mut Module, builtin: &Builtin) -> Result<(), WError> {
    let import_name = builtin.import_name();
    let names_section = module
        .names_section_mut()
        .expect("Names section not present");
    let function_names_section = match names_section {
        NameSection::Function(function_names_section) => function_names_section,
        _ => xbail!(WError::InternalError("Unexpected names section")),
    };
    prepend_function_name(function_names_section, import_name)?;
    Ok(())
}

fn patch_module(
    module: Module,
    builtins_names: &[&str],
) -> Result<(Module, PatchedBuiltinsMap), WError> {
    let mut module = module
        .parse_names()
        .map_err(|_| WError::InternalError("Unable to parse names"))?;

    let mut builtins: Vec<_> = builtins_names
        .iter()
        .map(|x| Builtin::new(x.to_string()))
        .collect();

    retain_only_used_builtins(&module, &mut builtins)?;
    add_function_type_id_to_builtins(&module, &mut builtins)?;

    add_import_section_if_missing(&mut module)?;
    for (builtin_idx, builtin) in builtins.iter_mut().enumerate() {
        prepend_builtin_to_import_section(&mut module, &builtin)?;
        prepend_builtin_to_names_section(&mut module, &builtin)?;
        shift_function_ids(&mut module, 1)?;
        let original_function_id = builtin.original_function_id.unwrap() + builtin_idx as u32 + 1;
        let new_function_id = 0;
        replace_function_id(&mut module, original_function_id, new_function_id)?;
    }

    let mut patched_builtins_map = PatchedBuiltinsMap::with_capacity(builtins.len());
    for builtin in builtins {
        patched_builtins_map.insert(builtin.name.clone(), builtin.import_name());
    }
    Ok((module, patched_builtins_map))
}

fn patch_file<P: AsRef<Path>>(
    path_in: P,
    path_out: P,
    builtins_names: &[&str],
) -> Result<PatchedBuiltinsMap, WError> {
    let module = parity_wasm::deserialize_file(path_in)?;
    let (patched_module, patched_builtins_map) = patch_module(module, builtins_names)?;
    elements::serialize_to_file(path_out, patched_module)?;
    Ok(patched_builtins_map)
}

fn main() -> Result<(), WError> {
    let config = Config::parse_cmdline()?;
    let symbols = symbols::extract_symbols(config.builtins_path)?;
    let builtins_names = symbols.builtins_names();
    let patched_builtins_map = patch_file(config.input_path, config.output_path, &builtins_names)?;
    if let Some(builtins_map_path) = config.builtins_map_path {
        patched_builtins_map.write_to_file(builtins_map_path, config.builtins_map_original_names)?;
    }
    Ok(())
}
