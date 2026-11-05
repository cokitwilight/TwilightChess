use std::mem::size_of;

use crate::{
    board::Move,
    engine::tt::{TTEntry, TTFlag, TTNodeType},
};

const TT_CLUSTER_SIZE: usize = 3;

const AGE_PENALTY: i32 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TTInsertResult {
    pub replaced: bool,
    pub collision: bool,
}

#[repr(C, align(32))]
#[derive(Clone, Debug, Default)]
pub struct TTCluster {
    entries: [Option<TTEntry>; TT_CLUSTER_SIZE],
}

#[derive(Clone, Debug)]
pub struct TranspositionTable {
    table: Box<[TTCluster]>,
    mask: usize,
    generation: u8,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let bytes = mb.max(1).saturating_mul(1024 * 1024);

        let cluster_size = size_of::<TTCluster>();
        let raw_clusters = (bytes / cluster_size).max(1);

        let cluster_count = floor_power_of_two(raw_clusters);

        let table = vec![TTCluster::default(); cluster_count];

        Self {
            table: table.into_boxed_slice(),
            mask: cluster_count - 1,
            generation: 1,
        }
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        key as usize & self.mask
    }

    #[inline(always)]
    /// Main probes require positive depth; quiescence can reuse either kind.
    pub fn get(&self, key: u64, node_type: TTNodeType) -> Option<&TTEntry> {
        self.get_any(key)
            .filter(|entry| entry.depth >= node_type as u8)
    }

    #[inline(always)]
    pub fn get_any(&self, key: u64) -> Option<&TTEntry> {
        let index = self.index(key);
        let cluster = &self.table[index];
        let entry_key = (key >> 48) as u16;

        cluster
            .entries
            .iter()
            .flatten()
            .find(|entry| entry.key == entry_key)
            .map(|entry| entry)
    }

    #[inline(always)]
    pub fn insert(
        &mut self,
        key: u64,
        is_pv: bool,
        bounds: TTFlag,
        depth: u8,
        eval: i16,
        static_eval: i16,
        best_move: Option<Move>,
    ) -> TTInsertResult {
        let index = self.index(key);
        let generation = self.generation;
        let cluster = &mut self.table[index];
        let entry_key = (key >> 48) as u16;

        let new_entry = TTEntry::new(
            entry_key,
            is_pv,
            bounds,
            generation,
            depth,
            eval,
            static_eval,
            best_move,
        );

        // Main and quiescence results for the same key share one entry.
        for old_entry in cluster.entries.iter_mut().flatten() {
            if old_entry.key != entry_key {
                continue;
            }

            // Depths = 0 is quiescence search
            let replaced = entry_value(&new_entry) >= entry_value(&old_entry);
            if replaced {
                *old_entry = new_entry;
            } else {
                // The old entry is better but still useful.
                old_entry.flag.update_generation(generation);
            }
            return TTInsertResult {
                replaced,
                collision: false,
            };
        }

        // Use an empty entry before evicting an entry.
        for entry in &mut cluster.entries {
            if entry.is_none() {
                *entry = Some(new_entry);
                return TTInsertResult::default();
            }
        }

        // Evict the least valuable entry from a full cluster.
        let replacement_index = cluster
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| {
                let entry = entry
                    .as_ref()
                    .expect("A full cluster should not contain an empty entry!");

                replacement_value(entry, generation)
            })
            .map(|(index, _)| index)
            .expect("TT cluster must have at least one entry");

        cluster.entries[replacement_index] = Some(new_entry);

        TTInsertResult {
            replaced: true,
            collision: true,
        }
    }

    pub fn new_search(&mut self) {
        self.generation = if self.generation >= 0x1F {
            1
        } else {
            self.generation + 1
        };
    }

    pub fn clear(&mut self) {
        self.table.fill(TTCluster::default());
    }

    pub fn len(&self) -> usize {
        self.table.len() * TT_CLUSTER_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn size_mb_approx(&self) -> usize {
        self.table.len() * size_of::<TTCluster>() / (1024 * 1024)
    }

    pub fn cluster_count(&self) -> usize {
        self.table.len()
    }

    pub fn generation(&self) -> u8 {
        self.generation
    }
}

fn floor_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }

    1usize << ((usize::BITS - 1 - n.leading_zeros()) as usize)
}

#[inline(always)]
fn entry_value(entry: &TTEntry) -> i32 {
    let exact_bonus = match entry.flag.bound_type() {
        TTFlag::Exact => 2,
        TTFlag::LowerBound | TTFlag::UpperBound => 0,
    };

    let pv_bonus = if entry.flag.is_pv() { 2 } else { 0 };

    i32::from(entry.depth) + exact_bonus + pv_bonus
}

#[inline(always)]
fn replacement_value(entry: &TTEntry, current_generation: u8) -> i32 {
    // let age = current_generation.wrapping_sub(entry.flag.generation()) as i32;
    let age = if entry.flag.generation() > current_generation {
        (current_generation as i32 + 31) - entry.flag.generation() as i32
    } else {
        current_generation as i32 - entry.flag.generation() as i32
    };

    entry_value(&entry) - age * AGE_PENALTY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::MoveType;
    use crate::types::PieceType;

    fn entry(depth: u8, is_pv: bool, bound: TTFlag) -> TTEntry {
        TTEntry::new(0, is_pv, bound, 0, depth, i16::from(depth), -123, None)
    }

    fn insert(table: &mut TranspositionTable, key: u64, entry: TTEntry) -> TTInsertResult {
        table.insert(
            key,
            entry.flag.is_pv(),
            entry.flag.bound_type(),
            entry.depth,
            entry.eval,
            entry.static_eval,
            entry.best_move,
        )
    }

    fn assert_entry_eq(actual: &TTEntry, expected: &TTEntry) {
        assert_eq!(actual.key, expected.key);
        assert_eq!(actual.flag.is_pv(), expected.flag.is_pv());
        assert_eq!(actual.flag.bound_type(), expected.flag.bound_type());
        assert_eq!(actual.flag.generation(), expected.flag.generation());
        assert_eq!(actual.depth, expected.depth);
        assert_eq!(actual.eval, expected.eval);
        assert_eq!(actual.static_eval, expected.static_eval);
        assert_eq!(actual.best_move, expected.best_move);
    }

    // Distinct high-16-bit signatures, all in cluster zero.
    fn key(signature: u16) -> u64 {
        u64::from(signature) << 48
    }

    #[test]
    fn cluster_packs_four_optional_entries_into_forty_bytes() {
        assert_eq!(size_of::<TTCluster>(), 40);
    }

    #[test]
    fn capacity_uses_power_of_two_clusters_within_the_memory_budget() {
        for mb in [0usize, 1, 2, 3, 8] {
            let table = TranspositionTable::new(mb);
            let budget = mb.max(1) * 1024 * 1024;
            let allocated = table.cluster_count() * size_of::<TTCluster>();

            assert!(table.cluster_count().is_power_of_two());
            assert!(allocated <= budget);
            assert!(allocated * 2 > budget);
            assert_eq!(table.len(), table.cluster_count() * 4);
            assert_eq!(table.size_mb_approx(), allocated / (1024 * 1024));
            assert!(!table.is_empty());
            assert_eq!(table.generation(), 1);
        }
    }

    #[test]
    fn empty_table_does_not_match_zero_or_maximum_signatures() {
        let table = TranspositionTable::new(1);
        for key in [0, 42, key(u16::MAX), u64::MAX] {
            assert!(table.get(key, TTNodeType::Main).is_none());
            assert!(table.get_any(key).is_none());
        }
    }

    #[test]
    fn insert_and_both_probes_preserve_every_field() {
        let promotion = Move::new(54, 63, MoveType::Capture, Some(PieceType::Queen));
        for is_pv in [false, true] {
            for bound in [TTFlag::Exact, TTFlag::LowerBound, TTFlag::UpperBound] {
                for best_move in [None, Some(promotion)] {
                    let mut table = TranspositionTable::new(1);
                    table.new_search();
                    for key in [0, u64::MAX] {
                        let expected = TTEntry::new(
                            (key >> 48) as u16,
                            is_pv,
                            bound,
                            table.generation(),
                            u8::MAX,
                            i16::MIN,
                            i16::MAX,
                            best_move,
                        );
                        assert_eq!(insert(&mut table, key, expected), TTInsertResult::default());
                        assert_entry_eq(table.get(key, TTNodeType::Main).unwrap(), &expected);
                        assert_entry_eq(table.get_any(key).unwrap(), &expected);
                        assert!(std::ptr::eq(
                            table.get(key, TTNodeType::Main).unwrap(),
                            table.get_any(key).unwrap(),
                        ));
                    }
                }
            }
        }
    }

    #[test]
    fn probes_check_the_signature_and_cluster_index() {
        let mut table = TranspositionTable::new(1);
        for signature in [0, 1, 2, u16::MAX] {
            assert_eq!(
                insert(&mut table, key(signature), entry(10, false, TTFlag::Exact)),
                TTInsertResult::default(),
            );
        }
        for signature in [0, 1, 2, u16::MAX] {
            let stored_key = key(signature);
            assert_eq!(
                table.get(stored_key, TTNodeType::Main).unwrap().key,
                signature
            );
            assert_eq!(table.get_any(stored_key).unwrap().key, signature);
            // Same signature in another cluster must miss.
            assert!(table.get(stored_key | 1, TTNodeType::Main).is_none());
            assert!(table.get_any(stored_key | 1).is_none());
        }
        // Same cluster, different signature must miss.
        assert!(table.get(key(3), TTNodeType::Main).is_none());
        assert!(table.get_any(key(3)).is_none());

        assert_eq!(
            insert(&mut table, key(1) | 1, entry(20, true, TTFlag::LowerBound)),
            TTInsertResult::default(),
        );
        assert_eq!(table.get(key(1), TTNodeType::Main).unwrap().depth, 10);
        assert_eq!(table.get(key(1) | 1, TTNodeType::Main).unwrap().depth, 20);
    }

    #[test]
    fn node_type_probes_include_depth_one_and_quiescence_depth_zero() {
        for depth in [0, 1, 255] {
            let mut table = TranspositionTable::new(1);
            insert(&mut table, key(42), entry(depth, false, TTFlag::Exact));
            assert_eq!(table.get(key(42), TTNodeType::Main).is_some(), depth > 0);
            assert_eq!(
                table.get(key(42), TTNodeType::Quiescence).unwrap().depth,
                depth
            );
            assert_eq!(table.get_any(key(42)).unwrap().depth, depth);
            for node_type in [TTNodeType::Main, TTNodeType::Quiescence] {
                assert!(table.get(key(43), node_type).is_none());
                assert!(table.get(key(42) | 1, node_type).is_none());
            }
        }
    }

    #[test]
    fn main_and_quiescence_entries_for_the_same_key_share_one_slot() {
        let mut table = TranspositionTable::new(1);
        let key = key(42);
        // Depth zero represents quiescence and is excluded by a main probe.
        assert_eq!(
            insert(&mut table, key, entry(0, false, TTFlag::Exact)),
            TTInsertResult::default(),
        );
        assert!(table.get(key, TTNodeType::Main).is_none());
        assert_eq!(table.get(key, TTNodeType::Quiescence).unwrap().depth, 0);
        assert_eq!(table.get_any(key).unwrap().depth, 0);

        assert_eq!(
            insert(&mut table, key, entry(10, true, TTFlag::LowerBound)),
            TTInsertResult {
                replaced: true,
                collision: false
            },
        );
        assert_eq!(table.get(key, TTNodeType::Main).unwrap().depth, 10);
        assert_eq!(table.get_any(key).unwrap().depth, 10);

        // A weaker quiescence result refreshes the retained main entry.
        table.new_search();
        assert_eq!(
            insert(&mut table, key, entry(0, false, TTFlag::Exact)),
            TTInsertResult::default(),
        );
        assert_eq!(table.get(key, TTNodeType::Main).unwrap().depth, 10);
        assert_eq!(
            table.get_any(key).unwrap().flag.generation(),
            table.generation()
        );
        assert_eq!(
            table.table[table.index(key)]
                .entries
                .iter()
                .flatten()
                .count(),
            1,
        );
    }

    #[test]
    fn matching_entry_keeps_quality_and_refreshes_only_its_generation() {
        let mut table = TranspositionTable::new(1);
        let key = key(42);
        let mut original = entry(10, true, TTFlag::Exact);
        original.key = 42;
        original.flag.update_generation(table.generation());
        original.best_move = Some(Move::new(12, 28, MoveType::Normal, None));
        insert(&mut table, key, original);
        table.new_search();

        // The exact and PV bonuses protect against a three-ply deeper bound.
        assert_eq!(
            insert(&mut table, key, entry(13, false, TTFlag::LowerBound)),
            TTInsertResult::default(),
        );
        original.flag.update_generation(table.generation());
        assert_entry_eq(table.get(key, TTNodeType::Main).unwrap(), &original);

        // Equal quality replaces the complete entry without a collision.
        let mut replacement = entry(14, false, TTFlag::UpperBound);
        replacement.key = 42;
        replacement.flag.update_generation(table.generation());
        assert_eq!(
            insert(&mut table, key, replacement),
            TTInsertResult {
                replaced: true,
                collision: false
            },
        );
        assert_entry_eq(table.get_any(key).unwrap(), &replacement);
    }

    #[test]
    fn full_cluster_eviction_accounts_for_age_exact_and_pv_bonuses() {
        let mut table = TranspositionTable::new(1);
        insert(&mut table, key(0), entry(10, false, TTFlag::LowerBound));
        table.new_search();
        for (signature, candidate) in [
            (1, entry(10, false, TTFlag::LowerBound)),
            (2, entry(10, false, TTFlag::Exact)),
            (3, entry(10, true, TTFlag::Exact)),
        ] {
            assert_eq!(
                insert(&mut table, key(signature), candidate),
                TTInsertResult::default()
            );
        }

        for (signature, victim) in [(4, 0), (5, 1), (6, 2)] {
            assert_eq!(
                insert(&mut table, key(signature), entry(30, false, TTFlag::Exact)),
                TTInsertResult {
                    replaced: true,
                    collision: true
                },
            );
            assert!(table.get(key(victim), TTNodeType::Main).is_none());
            assert!(table.get_any(key(victim)).is_none());
            assert!(table.get(key(signature), TTNodeType::Main).is_some());
        }
        assert!(table.get(key(3), TTNodeType::Main).unwrap().flag.is_pv());
    }

    #[test]
    fn clear_removes_all_entries_and_preserves_capacity_and_generation() {
        let mut table = TranspositionTable::new(1);
        for key in [0, key(1), key(2) | 1, u64::MAX] {
            insert(&mut table, key, entry(10, true, TTFlag::Exact));
        }
        table.new_search();
        let capacity = table.len();
        let generation = table.generation();
        table.clear();

        assert_eq!(table.len(), capacity);
        assert_eq!(table.generation(), generation);
        assert!(
            table
                .table
                .iter()
                .all(|cluster| cluster.entries.iter().all(Option::is_none))
        );
        for key in [0, key(1), key(2) | 1, u64::MAX] {
            assert!(table.get(key, TTNodeType::Main).is_none());
            assert!(table.get_any(key).is_none());
            assert_eq!(
                insert(&mut table, key, entry(0, false, TTFlag::LowerBound)),
                TTInsertResult::default(),
            );
        }
    }

    #[test]
    fn new_search_cycles_through_all_31_generations() {
        let mut table = TranspositionTable::new(1);
        for search in 0..93 {
            let expected_generation = (search % 31 + 1) as u8;
            assert_eq!(table.generation(), expected_generation);
            insert(&mut table, key(0), entry(10, false, TTFlag::Exact));
            assert_eq!(
                table
                    .get(key(0), TTNodeType::Main)
                    .unwrap()
                    .flag
                    .generation(),
                expected_generation
            );
            table.new_search();
        }
    }

    #[test]
    fn replacement_age_matches_elapsed_searches_for_every_generation() {
        let mut table = TranspositionTable::new(1);
        for stored_generation in 1..=31 {
            table.generation = stored_generation;
            let entry = TTEntry::new(
                0,
                false,
                TTFlag::LowerBound,
                stored_generation,
                255,
                0,
                0,
                None,
            );
            for elapsed_searches in 0..31 {
                assert_eq!(
                    replacement_value(&entry, table.generation()),
                    255 - elapsed_searches * 4,
                    "stored generation {stored_generation}, current generation {}, elapsed {elapsed_searches}",
                    table.generation(),
                );
                table.new_search();
            }
        }
    }

    #[test]
    fn replacement_age_counts_one_search_across_generation_wrap() {
        // Regression: wrapping_sub on u8 counts 31 -> 1 as 226 searches.
        let entry = TTEntry::new(0, false, TTFlag::LowerBound, 31, 20, 0, 0, None);
        assert_eq!(replacement_value(&entry, 1), 16);
    }

    #[test]
    fn generation_wrap_does_not_evict_a_deeper_recent_entry() {
        let mut table = TranspositionTable::new(1);
        for _ in 1..31 {
            table.new_search();
        }
        assert_eq!(table.generation(), 31);
        insert(&mut table, key(0), entry(20, false, TTFlag::LowerBound));
        table.new_search();
        assert_eq!(table.generation(), 1);
        for (signature, depth) in [(1, 1), (2, 8), (3, 9)] {
            insert(
                &mut table,
                key(signature),
                entry(depth, false, TTFlag::LowerBound),
            );
        }
        assert_eq!(
            insert(&mut table, key(4), entry(30, false, TTFlag::Exact)),
            TTInsertResult {
                replaced: true,
                collision: true
            },
        );

        // Age one leaves depth 20 more valuable than the fresh depth-one entry.
        assert!(
            table.get(key(0), TTNodeType::Main).is_some(),
            "the previous generation is only one search old"
        );
        assert!(table.get(key(1), TTNodeType::Main).is_none());
        for signature in [2, 3, 4] {
            assert!(table.get_any(key(signature)).is_some());
        }
    }
}
