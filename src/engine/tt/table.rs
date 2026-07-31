use std::mem::size_of;

use crate::engine::tt::{TTEntry, TTFlag, entry::TTNodeType};

const TT_CLUSTER_SIZE: usize = 4;

const AGE_PENALTY: i32 = 4;

pub trait TTReplace {
    fn depth(&self) -> u16;

    fn replacement_bonus(&self) -> i32 {
        0
    }

    fn replacement_class(&self) -> u8 {
        0
    }
}

impl TTReplace for TTEntry {
    #[inline(always)]
    fn depth(&self) -> u16 {
        self.depth
    }
    #[inline(always)]
    fn replacement_bonus(&self) -> i32 {
        let exact_bonus = match self.flag {
            TTFlag::Exact => 2,
            TTFlag::LowerBound | TTFlag::UpperBound => 0,
        };

        let node_bonus = match self.node_type {
            TTNodeType::Main => 8,
            TTNodeType::Quiescence => 0,
        };

        exact_bonus + node_bonus
    }

    #[inline(always)]
    fn replacement_class(&self) -> u8 {
        match self.node_type {
            TTNodeType::Quiescence => 0,
            TTNodeType::Main => 1,
        }
    }
}

#[derive(Clone, Debug)]
struct TTSlot<Entry> {
    key: u64,
    generation: u8,
    entry: Entry,
}

#[derive(Clone, Debug)]
struct TTCluster<Entry> {
    slots: [Option<TTSlot<Entry>>; TT_CLUSTER_SIZE],
}

impl<Entry> TTCluster<Entry> {
    fn empty() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TranspositionTable<Entry> {
    table: Box<[TTCluster<Entry>]>,
    mask: usize,
    generation: u8,
}

impl<Entry: TTReplace> TranspositionTable<Entry> {
    pub fn new(mb: usize) -> Self {
        let bytes = mb.max(1).saturating_mul(1024 * 1024);

        let cluster_size = size_of::<TTCluster<Entry>>().max(1);
        let raw_clusters = (bytes / cluster_size).max(1);

        let cluster_count = floor_power_of_two(raw_clusters);

        let mut table = Vec::with_capacity(cluster_count);
        table.resize_with(cluster_count, TTCluster::empty);

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
    pub fn get(&self, key: u64) -> Option<&Entry> {
        let index = self.index(key);
        let cluster = &self.table[index];

        for slot in &cluster.slots {
            if let Some(slot) = slot {
                if slot.key == key {
                    return Some(&slot.entry);
                }
            }
        }

        None
    }

    #[inline(always)]
    pub fn insert(&mut self, key: u64, entry: Entry) {
        let index = self.index(key);
        let generation = self.generation;
        let cluster = &mut self.table[index];

        for slot in &mut cluster.slots {
            let Some(old_slot) = slot else {
                continue;
            };

            if old_slot.key != key {
                continue;
            }

            if should_replace(&old_slot.entry, &entry) {
                *slot = Some(TTSlot {
                    key,
                    generation,
                    entry,
                });
            } else {
                // the old entry is better but still useful
                old_slot.generation = generation;
            }

            return;
        }

        // empty cluster

        for slot in &mut cluster.slots {
            if slot.is_none() {
                *slot = Some(TTSlot {
                    key,
                    generation,
                    entry,
                });
                return;
            }
        }

        // full cluster

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

        cluster.slots[replacement_index] = Some(TTSlot {
            key,
            generation,
            entry,
        });
    }

    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        for cluster in self.table.iter_mut() {
            for slot in &mut cluster.slots {
                *slot = None;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.table.len() * TT_CLUSTER_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn size_mb_approx(&self) -> usize {
        self.table.len() * size_of::<TTCluster<Entry>>() / (1024 * 1024)
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

fn should_replace<Entry: TTReplace>(old: &Entry, new: &Entry) -> bool {
    match new.replacement_class().cmp(&old.replacement_class()) {
        std::cmp::Ordering::Greater => {
            // Main-search entry replaces quiescence entry,
            // regardless of their numerically different depths.
            return true;
        }

        std::cmp::Ordering::Less => {
            // Quiescence entry must not overwrite a main-search entry.
            return false;
        }

        std::cmp::Ordering::Equal => {
            // Both entries come from the same search domain,
            // so their depths are comparable.
        }
    }

    let old_value = i32::from(old.depth()) + old.replacement_bonus();
    let new_value = i32::from(new.depth()) + new.replacement_bonus();

    new_value >= old_value
}

#[inline(always)]
fn replacement_value<Entry: TTReplace>(slot: &TTSlot<Entry>, current_generation: u8) -> i32 {
    let age = current_generation.wrapping_sub(slot.generation) as i32;

    i32::from(slot.entry.depth()) + slot.entry.replacement_bonus() - age * AGE_PENALTY
}
