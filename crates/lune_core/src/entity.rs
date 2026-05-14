use std::collections::BTreeSet;

use thiserror::Error;

pub type EntityResult<T> = std::result::Result<T, EntityError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    index: u32,
    generation: u32,
}

impl Entity {
    pub const NULL: Self = Self {
        index: u32::MAX,
        generation: 0,
    };

    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }

    pub const fn is_null(self) -> bool {
        self.generation == 0
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EntityError {
    #[error("entity handle uses reserved generation 0: index {index}")]
    NullEntity { index: u32 },

    #[error("entity index is not allocated: index {index}, generation {generation}")]
    InvalidEntity { index: u32, generation: u32 },

    #[error(
        "entity handle is stale: index {index}, handle generation {actual_generation}, current generation {expected_generation}"
    )]
    StaleEntity {
        index: u32,
        expected_generation: u32,
        actual_generation: u32,
    },

    #[error("entity slot is free: index {index}")]
    SlotFree { index: u32 },

    #[error("entity slot generation overflowed: index {index}")]
    GenerationOverflow { index: u32 },

    #[error("entity index space exhausted: capacity {capacity}")]
    IndexExhausted { capacity: u32 },
}

#[derive(Debug)]
struct EntitySlot {
    generation: u32,
    occupied: bool,
}

#[derive(Debug)]
pub struct EntitySlotTable {
    slots: Vec<EntitySlot>,
    free: BTreeSet<u32>,
    len: usize,
}

impl EntitySlotTable {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: BTreeSet::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free: BTreeSet::new(),
            len: 0,
        }
    }

    pub fn spawn(&mut self) -> EntityResult<Entity> {
        if let Some(index) = self.free.pop_first() {
            let slot = &mut self.slots[index as usize];
            slot.occupied = true;
            self.len += 1;
            return Ok(Entity {
                index,
                generation: slot.generation,
            });
        }

        if self.slots.len() >= u32::MAX as usize {
            return Err(EntityError::IndexExhausted { capacity: u32::MAX });
        }

        let next_index = self.slots.len() as u32;
        let slot = EntitySlot {
            generation: 1,
            occupied: true,
        };
        let entity = Entity {
            index: next_index,
            generation: slot.generation,
        };
        self.slots.push(slot);
        self.len += 1;

        Ok(entity)
    }

    pub fn despawn(&mut self, entity: Entity) -> EntityResult<()> {
        if entity.generation == Entity::NULL.generation {
            return Err(EntityError::NullEntity {
                index: entity.index,
            });
        }
        if (entity.index as usize) >= self.slots.len() {
            return Err(EntityError::InvalidEntity {
                index: entity.index,
                generation: entity.generation,
            });
        }

        let slot = &mut self.slots[entity.index as usize];
        if slot.generation != entity.generation {
            return Err(EntityError::StaleEntity {
                index: entity.index,
                expected_generation: slot.generation,
                actual_generation: entity.generation,
            });
        }

        if !slot.occupied {
            return Err(EntityError::SlotFree {
                index: entity.index,
            });
        }

        if let Some(new_generation) = slot.generation.checked_add(1) {
            slot.generation = new_generation;
            slot.occupied = false;
            self.free.insert(entity.index);
            self.len -= 1;
        } else {
            slot.occupied = false;
            self.len -= 1;
            return Err(EntityError::GenerationOverflow {
                index: entity.index,
            });
        }

        Ok(())
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        if entity.generation == 0 {
            return false;
        }
        let index = entity.index as usize;
        if index >= self.slots.len() {
            return false;
        }
        let slot = &self.slots[index];
        if !slot.occupied {
            return false;
        }
        if slot.generation != entity.generation {
            return false;
        }

        true
    }

    pub fn len(&self) -> usize {
        self.debug_assert_invariants();
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.debug_assert_invariants();
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.debug_assert_invariants();
        self.slots.capacity()
    }

    fn debug_assert_invariants(&self) {
        debug_assert!(self.len <= self.slots.len());
        debug_assert!(self.free.len() <= self.slots.len());

        #[cfg(debug_assertions)]
        {
            let occupied_slots = self.slots.iter().filter(|slot| slot.occupied).count();
            debug_assert_eq!(self.len, occupied_slots);

            for index in &self.free {
                let slot = &self.slots[*index as usize];
                debug_assert!(!slot.occupied);
            }

            for slot in &self.slots {
                debug_assert!(!slot.occupied || slot.generation != 0);
            }
        }
    }
}

impl Default for EntitySlotTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_seeded_at_generation(generation: u32) -> EntitySlotTable {
        EntitySlotTable {
            slots: vec![EntitySlot {
                generation,
                occupied: false,
            }],
            free: BTreeSet::from([0]),
            len: 0,
        }
    }

    #[test]
    fn generation_overflow_returns_overflow_error() {
        let mut table = table_seeded_at_generation(u32::MAX);
        let entity = table.spawn().unwrap();

        assert_eq!(entity.generation(), u32::MAX);

        assert_eq!(
            table.despawn(entity),
            Err(EntityError::GenerationOverflow {
                index: entity.index(),
            })
        );
    }

    #[test]
    fn overflowed_slot_is_dead_and_not_reused() {
        let mut table = table_seeded_at_generation(u32::MAX);
        let doomed = table.spawn().unwrap();

        assert_eq!(
            table.despawn(doomed),
            Err(EntityError::GenerationOverflow {
                index: doomed.index(),
            })
        );

        assert!(!table.is_alive(doomed));
        assert_eq!(table.len(), 0);

        let fresh = table.spawn().unwrap();
        assert_ne!(fresh.index(), doomed.index());
    }
}
