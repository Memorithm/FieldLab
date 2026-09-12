#![forbid(unsafe_code)]

use field_dynamics::{run_steps, IntegratorConfig};
use field_memory::PatternBank;
use std::error::Error;

const NODE_COUNT: usize = 16;
const MAX_FLIPS: usize = 4;
const EXPECTED_CASES: usize = 7_551;
const FIELD_STEPS: usize = 128;
const HOPFIELD_SWEEPS: usize = 32;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const CUE_TILT_RADIANS: f64 = 0.15;

#[derive(Clone, Copy, Debug, Default)]
struct MethodCounts {
    field: usize,
    hopfield: usize,
    nearest: usize,
    total: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let bank = PatternBank::new(reference_patterns())?;
    let model = bank.energy_model()?;
    let integrator = IntegratorConfig {
        dt: FIELD_DT,
        mobility: FIELD_MOBILITY,
    };
    let mut levels = [MethodCounts::default(); MAX_FLIPS + 1];

    for (target_index, target) in bank.patterns().iter().enumerate() {
        for mask in 0_u32..(1_u32 << NODE_COUNT) {
            let flips = usize::try_from(mask.count_ones()).expect("u32 popcount fits usize");
            if flips > MAX_FLIPS {
                continue;
            }

            let cue = corrupt(target, mask);
            let nearest_index = bank.nearest_neighbor_index(&cue)?;
            let hopfield = bank.hopfield_recover(&cue, HOPFIELD_SWEEPS)?;
            let field_initial = bank.encode_cue(&cue, CUE_TILT_RADIANS)?;
            let field_final = run_steps(field_initial, &model, integrator, FIELD_STEPS)?;
            let field = bank.decode_state(&field_final)?;

            let level = &mut levels[flips];
            level.total += 1;
            level.nearest += usize::from(nearest_index == target_index);
            level.hopfield += usize::from(hopfield == *target);
            level.field += usize::from(field == *target);
        }
    }

    let aggregate = levels.iter().copied().fold(MethodCounts::default(), add_counts);
    let replay_equal = replay_sentinel(&bank, &model, integrator)?;
    let zero_corruption_complete = levels[0].total == bank.patterns().len()
        && levels[0].field == levels[0].total
        && levels[0].hopfield == levels[0].total
        && levels[0].nearest == levels[0].total;
    let protocol_valid = aggregate.total == EXPECTED_CASES && zero_corruption_complete && replay_equal;
    let field_beats_hopfield = aggregate.field > aggregate.hopfield;
    let field_beats_nearest = aggregate.field > aggregate.nearest;
    let field_exact_through_three = levels[..=3]
        .iter()
        .all(|level| level.field == level.total);

    let manifest = format!(
        "fl1|nodes={NODE_COUNT}|patterns=3|max_flips={MAX_FLIPS}|field_steps={FIELD_STEPS}|dt={FIELD_DT:.17}|mobility={FIELD_MOBILITY:.17}|tilt={CUE_TILT_RADIANS:.17}|hopfield_sweeps={HOPFIELD_SWEEPS}|mask_order=ascending|tie_break=bank_order"
    );
    let provenance_fingerprint = fnv1a64(manifest.as_bytes());

    println!("{{");
    println!("  \"experiment\": \"FL-1\",");
    println!("  \"protocol\": \"associative-recall-v1\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{provenance_fingerprint:016x}\",");
    println!("  \"node_count\": {NODE_COUNT},");
    println!("  \"stored_patterns\": {},", bank.patterns().len());
    println!("  \"max_flips\": {MAX_FLIPS},");
    println!("  \"total_cases\": {},", aggregate.total);
    println!("  \"field_steps\": {FIELD_STEPS},");
    println!("  \"field_dt\": {FIELD_DT:.17},");
    println!("  \"cue_tilt_radians\": {CUE_TILT_RADIANS:.17},");
    println!("  \"hopfield_sweeps\": {HOPFIELD_SWEEPS},");
    println!("  \"levels\": [");
    for (index, level) in levels.iter().enumerate() {
        let suffix = if index == MAX_FLIPS { "" } else { "," };
        println!(
            "    {{\"flips\": {index}, \"cases\": {}, \"field_exact\": {}, \"field_rate\": {:.9}, \"hopfield_exact\": {}, \"hopfield_rate\": {:.9}, \"nearest_exact\": {}, \"nearest_rate\": {:.9}}}{suffix}",
            level.total,
            level.field,
            rate(level.field, level.total),
            level.hopfield,
            rate(level.hopfield, level.total),
            level.nearest,
            rate(level.nearest, level.total),
        );
    }
    println!("  ],");
    println!(
        "  \"aggregate\": {{\"field_exact\": {}, \"field_rate\": {:.9}, \"hopfield_exact\": {}, \"hopfield_rate\": {:.9}, \"nearest_exact\": {}, \"nearest_rate\": {:.9}}},",
        aggregate.field,
        rate(aggregate.field, aggregate.total),
        aggregate.hopfield,
        rate(aggregate.hopfield, aggregate.total),
        aggregate.nearest,
        rate(aggregate.nearest, aggregate.total),
    );
    println!("  \"replay_equal\": {replay_equal},");
    println!("  \"zero_corruption_complete\": {zero_corruption_complete},");
    println!("  \"field_beats_hopfield_aggregate\": {field_beats_hopfield},");
    println!("  \"field_beats_nearest_aggregate\": {field_beats_nearest},");
    println!("  \"field_exact_through_three_flips\": {field_exact_through_three},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");

    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn reference_patterns() -> Vec<Vec<i8>> {
    vec![
        vec![1; NODE_COUNT],
        vec![1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1, -1, -1, -1],
        vec![1, 1, 1, 1, -1, -1, -1, -1, 1, 1, 1, 1, -1, -1, -1, -1],
    ]
}

fn corrupt(pattern: &[i8], mask: u32) -> Vec<i8> {
    pattern
        .iter()
        .enumerate()
        .map(|(index, symbol)| {
            if mask & (1_u32 << index) == 0 {
                *symbol
            } else {
                -*symbol
            }
        })
        .collect()
}

fn add_counts(mut left: MethodCounts, right: MethodCounts) -> MethodCounts {
    left.field += right.field;
    left.hopfield += right.hopfield;
    left.nearest += right.nearest;
    left.total += right.total;
    left
}

fn rate(successes: usize, total: usize) -> f64 {
    let successes_u32 = u32::try_from(successes).expect("FL-1 count fits u32");
    let total_u32 = u32::try_from(total).expect("FL-1 count fits u32");
    f64::from(successes_u32) / f64::from(total_u32)
}

fn replay_sentinel(
    bank: &PatternBank,
    model: &field_core::EnergyModel,
    integrator: IntegratorConfig,
) -> Result<bool, Box<dyn Error>> {
    let target = &bank.patterns()[0];
    let cue = corrupt(target, 0b1111);
    let initial = bank.encode_cue(&cue, CUE_TILT_RADIANS)?;
    let first = run_steps(initial.clone(), model, integrator, FIELD_STEPS)?;
    let second = run_steps(initial, model, integrator, FIELD_STEPS)?;
    Ok(first == second)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_patterns_are_sixteen_symbols() {
        assert!(reference_patterns()
            .iter()
            .all(|pattern| pattern.len() == NODE_COUNT));
    }

    #[test]
    fn corruption_mask_flips_declared_positions() {
        let pattern = vec![1; NODE_COUNT];
        let corrupted = corrupt(&pattern, 0b0101);
        assert_eq!(corrupted[0], -1);
        assert_eq!(corrupted[1], 1);
        assert_eq!(corrupted[2], -1);
    }

    #[test]
    fn expected_case_count_matches_combinatorics() {
        let cases_per_pattern = (0_u32..(1_u32 << NODE_COUNT))
            .filter(|mask| usize::try_from(mask.count_ones()).unwrap() <= MAX_FLIPS)
            .count();
        assert_eq!(cases_per_pattern * 3, EXPECTED_CASES);
    }
}
