use std::mem::size_of;

use crate::engine::tt::{TTEntry, TTFlag, TTNodeType};

const TT_CLUSTER_SIZE: usize = 4;

const AGE_PENALTY: i32 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TTInsertResult {
    pub replaced: bool,
    pub collision: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TTSlot {
    key: u64,
    generation: u8,
    entry: TTEntry,
}

#[derive(Clone, Debug, Default)]
pub struct TTCluster {
    slots: [Option<TTSlot>; TT_CLUSTER_SIZE],
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
    pub fn get(&self, key: u64, node_type: TTNodeType) -> Option<&TTEntry> {
        let index = self.index(key);
        let cluster = &self.table[index];

        cluster
            .slots
            .iter()
            .flatten()
            .find(|slot| slot.key == key && slot.entry.node_type == node_type)
            .map(|slot| &slot.entry)
    }

    #[inline(always)]
    pub fn get_any(&self, key: u64) -> Option<&TTEntry> {
        let index = self.index(key);
        let cluster = &self.table[index];

        cluster
            .slots
            .iter()
            .flatten()
            .find(|slot| slot.key == key)
            .map(|slot| &slot.entry)
    }

    #[inline(always)]
    pub fn insert(&mut self, key: u64, entry: TTEntry) -> TTInsertResult {
        let index = self.index(key);
        let generation = self.generation;
        let cluster = &mut self.table[index];

        let new_slot = TTSlot {
            key,
            generation,
            entry,
        };

        // Entries for the same key and node type share a slot.
        for old_slot in cluster.slots.iter_mut().flatten() {
            if old_slot.key != key || old_slot.entry.node_type != entry.node_type {
                continue;
            }

            // Depths are comparable because both entries have the same node type.
            let replaced = entry_value(&entry) >= entry_value(&old_slot.entry);
            if replaced {
                *old_slot = new_slot;
            } else {
                // The old entry is better but still useful.
                old_slot.generation = generation;
            }
            return TTInsertResult {
                replaced,
                collision: false,
            };
        }

        // Use an empty slot before evicting an entry.
        for slot in &mut cluster.slots {
            if slot.is_none() {
                *slot = Some(new_slot);
                return TTInsertResult::default();
            }
        }

        // Evict the least valuable entry from a full cluster.
        let replacement_index = cluster
            .slots
            .iter()
            .enumerate()
            .min_by_key(|(_, slot)| {
                let slot = slot
                    .as_ref()
                    .expect("A full cluster should not contain an empty slot!");

                replacement_value(slot, generation)
            })
            .map(|(index, _)| index)
            .expect("TT cluster must have at least one slot");

        cluster.slots[replacement_index] = Some(new_slot);

        TTInsertResult {
            replaced: true,
            collision: true,
        }
    }

    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
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
    let exact_bonus = match entry.flag {
        TTFlag::Exact => 2,
        TTFlag::LowerBound | TTFlag::UpperBound => 0,
    };
    let node_bonus = match entry.node_type {
        TTNodeType::Main => 8,
        TTNodeType::Quiescence => 0,
    };

    i32::from(entry.depth) + exact_bonus + node_bonus
}

#[inline(always)]
fn replacement_value(slot: &TTSlot, current_generation: u8) -> i32 {
    let age = current_generation.wrapping_sub(slot.generation) as i32;

    entry_value(&slot.entry) - age * AGE_PENALTY
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(depth: u16, node_type: TTNodeType, flag: TTFlag) -> TTEntry {
        TTEntry {
            depth,
            eval: i32::from(depth),
            best_move: None,
            flag,
            node_type,
        }
    }

    #[test]
    fn same_key_can_store_both_node_types() {
        let mut table = TranspositionTable::new(1);
        let key = 42;

        assert_eq!(
            table.insert(key, entry(10, TTNodeType::Main, TTFlag::LowerBound)),
            TTInsertResult::default()
        );
        assert_eq!(
            table.insert(key, entry(20, TTNodeType::Quiescence, TTFlag::Exact)),
            TTInsertResult::default()
        );

        assert_eq!(table.get(key, TTNodeType::Main).unwrap().depth, 10);
        assert_eq!(table.get(key, TTNodeType::Quiescence).unwrap().depth, 20);
        assert_eq!(table.get_any(key).unwrap().node_type, TTNodeType::Main);
        assert!(table.get_any(key + 1).is_none());

        let capacity = table.len();
        table.clear();
        assert!(table.get_any(key).is_none());
        assert!(table.get(key, TTNodeType::Quiescence).is_none());
        assert_eq!(table.len(), capacity);
    }

    #[test]
    fn matching_entry_keeps_quality_and_refreshes_its_generation() {
        let mut table = TranspositionTable::new(1);
        let key = 42;
        table.insert(key, entry(10, TTNodeType::Main, TTFlag::Exact));
        table.new_search();

        // The exact-score bonus protects against a slightly deeper bound.
        assert_eq!(
            table.insert(key, entry(11, TTNodeType::Main, TTFlag::LowerBound)),
            TTInsertResult::default()
        );
        assert_eq!(table.get(key, TTNodeType::Main).unwrap().depth, 10);
        let slot = table.table[table.index(key)].slots[0].unwrap();
        assert_eq!(slot.generation, table.generation());

        // Equal quality replaces the old entry without recording a collision.
        assert_eq!(
            table.insert(key, entry(12, TTNodeType::Main, TTFlag::UpperBound)),
            TTInsertResult {
                replaced: true,
                collision: false,
            }
        );
        assert_eq!(table.get(key, TTNodeType::Main).unwrap().depth, 12);
    }

    #[test]
    fn full_cluster_eviction_accounts_for_depth_bonuses_and_wrapping_age() {
        let mut table = TranspositionTable::new(1);
        let stride = table.cluster_count() as u64;
        table.generation = u8::MAX;
        table.insert(0, entry(10, TTNodeType::Main, TTFlag::LowerBound));
        table.new_search();
        assert_eq!(table.generation(), 0);

        for (key, candidate) in [
            (stride, entry(10, TTNodeType::Main, TTFlag::LowerBound)),
            (2 * stride, entry(10, TTNodeType::Main, TTFlag::Exact)),
            (3 * stride, entry(10, TTNodeType::Quiescence, TTFlag::Exact)),
        ] {
            assert_eq!(table.insert(key, candidate), TTInsertResult::default());
        }

        let candidate = entry(30, TTNodeType::Main, TTFlag::Exact);
        let collision = TTInsertResult {
            replaced: true,
            collision: true,
        };
        // The main-search bonus outweighs the quiescence exact-score bonus.
        assert_eq!(table.insert(4 * stride, candidate), collision);
        assert!(table.get_any(3 * stride).is_none());
        // Age breaks the tie between otherwise identical main-search entries.
        assert_eq!(table.insert(5 * stride, candidate), collision);
        assert!(table.get_any(0).is_none());
        // The exact-score bonus protects the next entry at the same depth.
        assert_eq!(table.insert(6 * stride, candidate), collision);
        assert!(table.get_any(stride).is_none());
        assert!(table.get_any(2 * stride).is_some());
    }
}
