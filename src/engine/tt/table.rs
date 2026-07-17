use std::mem::size_of;

use crate::engine::tt::TTEntry;

pub trait TTReplace {
    fn depth(&self) -> usize;
}

impl TTReplace for TTEntry {
    fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Clone, Debug)]
struct TTSlot<Entry> {
    key: u64,
    entry: Entry,
}

#[derive(Clone, Debug)]
pub struct TranspositionTable<Entry> {
    table: Box<[Option<TTSlot<Entry>>]>,
    mask: usize,
}

impl<Entry: TTReplace> TranspositionTable<Entry> {
    pub fn new(mb: usize) -> Self {
        let bytes = mb.max(1).saturating_mul(1024 * 1024);

        let slot_size = size_of::<Option<TTSlot<Entry>>>().max(1);
        let raw_entries = (bytes / slot_size).max(1);

        let entries = floor_power_of_two(raw_entries);

        let mut table = Vec::with_capacity(entries);
        table.resize_with(entries, || None);

        Self {
            table: table.into_boxed_slice(),
            mask: entries - 1,
        }
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        key as usize & self.mask
    }

    #[inline(always)]
    pub fn get(&self, key: u64) -> Option<&Entry> {
        let index = self.index(key);

        match &self.table[index] {
            Some(slot) if slot.key == key => Some(&slot.entry),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, key: u64, entry: Entry) {
        let index = self.index(key);

        let should_replace = match &self.table[index] {
            None => true,

            Some(old_slot) if old_slot.key == key => {
                // Same position. Prefer newer result if it is at least as deep.
                entry.depth() >= old_slot.entry.depth()
            }

            Some(old_slot) => {
                // Collision with a different position.
                // Keep the deeper entry.
                entry.depth() >= old_slot.entry.depth()
            }
        };

        if should_replace {
            self.table[index] = Some(TTSlot { key, entry });
        }
    }

    pub fn clear(&mut self) {
        for slot in self.table.iter_mut() {
            *slot = None;
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn size_mb_approx(&self) -> usize {
        self.table.len() * size_of::<Option<TTSlot<Entry>>>() / (1024 * 1024)
    }
}

fn floor_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }

    1usize << ((usize::BITS - 1 - n.leading_zeros()) as usize)
}
