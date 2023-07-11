use super::list;

use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::io::{BufWriter, Write};

use std::{env, fs::File, path::Path, str};

use libmqm_sys::lib as mqsys;


pub fn name_filter(value: mqsys::MQLONG, name: &str, str_fn: list::MqCStrFn) -> bool {
    unsafe { str::from_utf8_unchecked(std::ffi::CStr::from_ptr(str_fn(value)).to_bytes()) == name }
}

/// Load the `MQI_BY_NAME_STR` into a Vec
fn by_name(by_name_mqi: &[mqsys::MQI_BY_NAME_STR]) -> Vec<(&str, i32)> {
    by_name_mqi
        .iter()
        .map(|entry| {
            (
                unsafe { str::from_utf8_unchecked(CStr::from_ptr(entry.name).to_bytes()) },
                entry.value,
            )
        })
        .filter(|(name, ..)| !name.is_empty())
        .collect()
}

/// Load the `MQI_BY_VALUE_STR` into a Vec
fn by_value(by_value_mqi: &[mqsys::MQI_BY_VALUE_STR]) -> Vec<(i32, &str)> {
    by_value_mqi
        .iter()
        .map(|entry| {
            (entry.value, unsafe {
                str::from_utf8_unchecked(CStr::from_ptr(entry.name).to_bytes())
            })
        })
        .filter(|(.., name)| !name.is_empty())
        .collect()
}

fn as_array(by_value: &[&(mqsys::MQLONG, &str)]) -> String {
    let mut result = String::new();
    result.push('[');
    for (value, name) in by_value {
        result.push_str(&format!("({value},\"{name}\"),"));
    }
    result.push(']');
    result
}

fn as_phf(by_value: &[&(mqsys::MQLONG, &str)]) -> String {
    let mut phf_set = phf_codegen::Map::<mqsys::MQLONG>::new();
    for (value, name) in by_value {
        phf_set.entry(*value, &format!("\"{name}\""));
    }
    phf_set.build().to_string()
}

pub fn generate() {
    let path = Path::new(&env::var("OUT_DIR").expect("OUT_DIR is mandatory for builds")).join("mqconstants.rs");
    let mut file = BufWriter::new(File::create(path).expect("Failure to create mqconstants.rs"));

    let by_name_mqi = unsafe { mqsys::MQI_BY_NAME_STR };
    let by_name = by_name(&by_name_mqi);

    let by_value_mqi = unsafe { mqsys::MQI_BY_VALUE_STR };
    let by_value = by_value(&by_value_mqi);

    // Gather the list of constants for each prefix by using
    // the _STR c functions and CONSTANTS which was derived from
    // the header file
    let primary_constants = list::CONSTANTS
        .iter()
        .map(|(prefix, check)| {
            let mut by_value_set: Vec<_> = by_value
                .iter()
                .filter(|(value, name)| name_filter(*value, name, *check))
                .collect();
            by_value_set.sort_by_key(|(k, ..)| *k);
            (*prefix, by_value_set)
        })
        .collect::<HashMap<_, _>>();

    // Collect a list of constants that are assigned to a prefix
    let primary_set = primary_constants
        .values()
        .flatten()
        .map(|(.., name)| *name)
        .collect::<HashSet<_>>();

    // List of unassigned constants
    let unassigned_constants = by_value
        .iter()
        .filter(|(.., name)| !primary_set.contains(name))
        .filter(|(.., name)| {
            // Ignore some constants that are used for MQI structures
            // and ranges
            !name.contains("_LENGTH")
                && !name.contains("_VERSION")
                && !name.ends_with("_LAST")
                && !name.ends_with("_FIRST")
                && !name.ends_with("_LAST_USED")
        })
        .collect::<Vec<_>>();

    // Create a map of primary and extra constants
    let mut prefix_constants = primary_constants
        .iter()
        .map(|(prefix, primary)| {
            // Similar prefixes ie prefixes that start with another prefix.
            // This need to be excluded from the "extra" list
            let similar: HashSet<_> = primary_constants
                .iter()
                .filter_map(|(&other_prefix, ..)| {
                    (*prefix != other_prefix && other_prefix.starts_with(prefix)).then_some(other_prefix)
                })
                .collect();
            // 'extra' are the constants that were _not_ yielded from the _STR c functions
            // They are still useful
            let extra: Vec<_> = unassigned_constants
                .iter()
                .filter(|(.., name)| {
                    name.starts_with(prefix) && !similar.iter().any(|&other_prefix| name.starts_with(other_prefix))
                })
                .copied()
                .collect();
            (*prefix, (primary, extra))
        })
        .collect::<Vec<_>>();

    prefix_constants.sort_by_key(|&(prefix, ..)| prefix);

    // Pick a lookup type based on the size of the constants for a prefix
    // TODO: Determine best ranges for performance
    for (prefix, (primary, ref extra)) in prefix_constants {
        write!(&mut file, "pub const {prefix}CONST: ").unwrap();
        match primary.len() {
            0..=63 => {
                // Linear search array
                writeln!(
                    &mut file,
                    "LinearSource = ConstSource(&{}, &{});",
                    as_array(primary),
                    as_array(extra)
                )
                .unwrap();
            }
            64..=255 => {
                // Binary search array
                writeln!(
                    &mut file,
                    "BinarySearchSource = ConstSource(BinarySearch(&{}), &{});",
                    as_array(primary),
                    as_array(extra)
                )
                .unwrap();
            }
            _ => {
                // Perfect hash used for larger constant lists
                writeln!(
                    &mut file,
                    "PhfSource = ConstSource(&{}, &{});",
                    as_phf(primary),
                    as_array(extra)
                )
                .unwrap();
            }
        }
    }

    // Full MQI_BY_STRING
    let mut mqi_by_string = phf_codegen::Map::<&str>::new();
    for (name, value) in by_name {
        mqi_by_string.entry(name, &value.to_string());
    }
    writeln!(
        &mut file,
        "pub(crate) const MQI_BY_STRING: ::phf::Map<&'static str, ::libmqm_sys::lib::MQLONG> = {};",
        mqi_by_string.build()
    )
    .unwrap();
}
