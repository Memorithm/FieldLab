use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Calibration,
    Holdout,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct PartitionRecord {
    case_id: String,
    sha256: String,
    partition: Partition,
}

fn digest_hex(digest: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(digest.len() * 2);
    for &byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn partition_case(case_id: &str) -> Result<PartitionRecord, &'static str> {
    if case_id.is_empty() {
        return Err("case identifier must not be empty");
    }

    let digest = Sha256::digest(case_id.as_bytes());
    let partition = if digest[0] < 0x80 {
        Partition::Calibration
    } else {
        Partition::Holdout
    };

    Ok(PartitionRecord {
        case_id: case_id.to_owned(),
        sha256: digest_hex(&digest),
        partition,
    })
}

fn partition_cases(case_ids: &[String]) -> Result<Vec<PartitionRecord>, &'static str> {
    let mut seen = BTreeSet::new();
    let mut records = Vec::with_capacity(case_ids.len());

    for case_id in case_ids {
        if !seen.insert(case_id.as_str()) {
            return Err("duplicate case identifier is not allowed");
        }
        records.push(partition_case(case_id)?);
    }

    Ok(records)
}

fn main() {
    let case_ids: Vec<String> = env::args().skip(1).collect();
    if case_ids.is_empty() {
        eprintln!("usage: cargo run -p field-bench --bin fl5_partition -- <case-id> [case-id ...]");
        std::process::exit(2);
    }

    let records = match partition_cases(&case_ids) {
        Ok(records) => records,
        Err(message) => {
            eprintln!("invalid FL-5 case set: {message}");
            std::process::exit(2);
        }
    };

    match serde_json::to_string_pretty(&records) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("failed to serialize FL-5 partition records: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Partition, partition_case, partition_cases};

    #[test]
    fn partition_is_stable_for_known_case_ids() {
        let calibration = partition_case("case-a").unwrap();
        assert_eq!(
            calibration.sha256,
            "178f13edf91be156a03cf2413ba6e064fd097fe73d010d326965b11153333002"
        );
        assert_eq!(calibration.partition, Partition::Calibration);

        let holdout = partition_case("case-b").unwrap();
        assert_eq!(
            holdout.sha256,
            "8f22e6738f402d830725703f57b60ef3409e01b32d0e9f5dc6a78a772b3cb082"
        );
        assert_eq!(holdout.partition, Partition::Holdout);
    }

    #[test]
    fn empty_case_id_is_rejected() {
        assert_eq!(partition_case(""), Err("case identifier must not be empty"));
    }

    #[test]
    fn duplicate_case_ids_are_rejected_fail_closed() {
        let cases = vec!["case-a".to_owned(), "case-b".to_owned(), "case-a".to_owned()];
        assert_eq!(
            partition_cases(&cases),
            Err("duplicate case identifier is not allowed")
        );
    }

    #[test]
    fn unique_case_set_preserves_declared_order() {
        let cases = vec!["case-b".to_owned(), "case-a".to_owned()];
        let records = partition_cases(&cases).unwrap();
        assert_eq!(records[0].case_id, "case-b");
        assert_eq!(records[1].case_id, "case-a");
    }
}
