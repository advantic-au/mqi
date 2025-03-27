use super::list;

use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::io;
use std::str;

use libmqm_sys::lib as mqsys;
// Load the `MQI_BY_NAME_STR` into a Vec
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
    use std::fmt::Write as _;
    let mut result = String::new();
    result.push('[');
    for (value, name) in by_value {
        let _ = write!(result, "({value},\"{name}\"),");
    }
    result.push(']');
    result
}

fn as_phf(by_value: &[&(mqsys::MQLONG, &str)]) -> String {
    let mut phf_set = phf_codegen::Map::new();
    for (value, name) in by_value {
        phf_set.entry(*value, &format!("\"{name}\""));
    }
    phf_set.build().to_string()
}

pub fn generate_constants(w: &mut impl std::io::Write) -> Result<(), io::Error> {
    let by_name_mqi = unsafe { mqsys::MQI_BY_NAME_STR };
    let by_name = by_name(&by_name_mqi);

    let by_value_mqi = unsafe { mqsys::MQI_BY_VALUE_STR };
    let by_value = by_value(&by_value_mqi);

    // Gather the list of constants for each prefix by using
    // the _STR c functions and CONSTANTS which was derived from
    // the header file
    let primary_constants = list::all_constants()
        .map(|(prefix, check)| {
            let mut by_value_set: Vec<_> = by_value
                .iter()
                .filter(|(value, name)| unsafe { str::from_utf8_unchecked(check(*value).to_bytes()) == *name })
                .collect();
            by_value_set.sort_by_key(|(k, ..)| *k);
            (prefix, by_value_set)
        })
        .chain(list::PREFIX_CONSTANTS.iter().map(|prefix| {
            (
                *prefix,
                by_value.iter().filter(|(_, name)| name.starts_with(prefix)).collect(),
            )
        }))
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

    // Show the unassigned constants without a _str function
    // dbg!(unassigned_constants.iter().filter(|(_, name)| {
    //     !all_constants().any(|(prefix, _)| name.starts_with(prefix))
    // }).collect::<Vec<_>>());
    // panic!();

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
                    name.starts_with(prefix) && !similar.iter().any(|other_prefix| name.starts_with(other_prefix))
                })
                .copied()
                .collect();
            (*prefix, (primary, extra))
        })
        .collect::<Vec<_>>();

    prefix_constants.sort_by_key(|(prefix, ..)| *prefix);

    // Pick a lookup type based on the size of the constants for a prefix
    // TODO: Determine best ranges for performance
    for (prefix, (primary, ref extra)) in prefix_constants {
        write!(w, "pub const {prefix}CONST: ")?;
        match primary.len() {
            0..=63 => {
                // Linear search array
                writeln!(w, "LinearSource = ConstSource(&{}, &{});", as_array(primary), as_array(extra))?;
            }
            64..=255 => {
                // Binary search array
                writeln!(
                    w,
                    "BinarySearchSource = ConstSource(BinarySearch(&{}), &{});",
                    as_array(primary),
                    as_array(extra)
                )?;
            }
            _ => {
                // Perfect hash used for larger constant lists
                writeln!(w, "PhfSource = ConstSource(&{}, &{});", as_phf(primary), as_array(extra))?;
            }
        }
    }

    // Full MQI_BY_STRING
    let mut mqi_by_string = phf_codegen::Map::<&str>::new();
    for (name, value) in by_name {
        mqi_by_string.entry(name, &value.to_string());
    }
    writeln!(
        w,
        "pub(crate) const MQI_BY_STRING: ::phf::Map<&'static str, ::libmqm_sys::lib::MQLONG> = {};",
        mqi_by_string.build()
    )
}
