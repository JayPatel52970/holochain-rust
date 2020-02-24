use crate::{
    wasm_engine::{Runtime},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    self,
    dna::Dna,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
};
use holochain_persistence_api::cas::content::Address;
use holochain_persistence_api::cas::content::AddressableContent;
use holochain_wasmer_host::*;
use std::str::FromStr;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn get_entry_type(dna: &Dna, entry_type_name: &str) -> Result<EntryType, HolochainError> {
    let entry_type = EntryType::from_str(&entry_type_name.to_string())
        .map_err(|_| WasmError::UnknownEntryType)?;

    // Check if AppEntry is a valid AppEntryType
    if entry_type.is_app() {
        let result = dna.get_entry_type_def(entry_type_name);
        if result.is_none() {
            Err(WasmError::UnknownEntryType)?;
        }
    }
    // Done
    Ok(entry_type)
}

/// ZomeApiFunction::entry_address function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: entry_type_name and entry_value as JsonString
/// Returns an HcApiReturnCode as I64
pub fn invoke_entry_address(runtime: &mut Runtime, entry: Entry) -> Result<Address, HolochainError> {
    // Check if entry_type is valid
    let dna = runtime
        .context().map_err(|e| WasmError::Zome(e.to_string()))?
        .state()
        .unwrap()
        .nucleus()
        .dna()
        .expect("Should have DNA");

    // bail if error
    get_entry_type(&dna, &entry.entry_type().to_string())?;

    // Return result
    Ok(entry.address())
}
