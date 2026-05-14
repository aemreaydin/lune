use lune_core::{Entity, EntityError, EntitySlotTable};

#[test]
fn entity_handle_exposes_index_generation_and_reserved_null() {
    let entity = Entity::new(7, 3);

    assert_eq!(entity.index(), 7);
    assert_eq!(entity.generation(), 3);
    assert!(!entity.is_null());

    assert_eq!(Entity::NULL.generation(), 0);
    assert!(Entity::NULL.is_null());
    assert!(Entity::new(9, 0).is_null());
}

#[test]
fn spawn_creates_a_live_entity() {
    let mut table = EntitySlotTable::new();

    let entity = table.spawn().unwrap();

    assert_eq!(entity.generation(), 1);
    assert!(table.is_alive(entity));
    assert_eq!(table.len(), 1);
    assert!(!table.is_empty());
}

#[test]
fn despawn_invalidates_the_old_handle() {
    let mut table = EntitySlotTable::new();
    let entity = table.spawn().unwrap();

    table.despawn(entity).unwrap();

    assert!(!table.is_alive(entity));
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
}

#[test]
fn despawning_the_same_handle_twice_is_rejected() {
    let mut table = EntitySlotTable::new();
    let entity = table.spawn().unwrap();

    table.despawn(entity).unwrap();

    assert_eq!(
        table.despawn(entity),
        Err(EntityError::StaleEntity {
            index: entity.index(),
            expected_generation: entity.generation() + 1,
            actual_generation: entity.generation(),
        })
    );
}

#[test]
fn recycled_slot_reuses_index_and_increments_generation() {
    let mut table = EntitySlotTable::new();
    let first = table.spawn().unwrap();

    table.despawn(first).unwrap();
    let reused = table.spawn().unwrap();

    assert_eq!(reused.index(), first.index());
    assert_eq!(reused.generation(), first.generation() + 1);
    assert!(!table.is_alive(first));
    assert!(table.is_alive(reused));
}

#[test]
fn stale_handle_cannot_despawn_reused_slot() {
    let mut table = EntitySlotTable::new();
    let first = table.spawn().unwrap();

    table.despawn(first).unwrap();
    let reused = table.spawn().unwrap();

    assert_eq!(
        table.despawn(first),
        Err(EntityError::StaleEntity {
            index: first.index(),
            expected_generation: reused.generation(),
            actual_generation: first.generation(),
        })
    );
    assert!(table.is_alive(reused));
}

#[test]
fn generation_zero_handles_are_rejected() {
    let mut table = EntitySlotTable::new();
    let null = Entity::new(0, 0);

    assert_eq!(
        table.despawn(null),
        Err(EntityError::NullEntity {
            index: null.index(),
        })
    );
    assert!(!table.is_alive(null));
}

#[test]
fn foreign_or_unallocated_index_is_rejected() {
    let mut table = EntitySlotTable::new();
    let foreign = Entity::new(42, 1);

    assert_eq!(
        table.despawn(foreign),
        Err(EntityError::InvalidEntity {
            index: 42,
            generation: 1,
        })
    );
    assert!(!table.is_alive(foreign));
}

#[test]
fn synthesized_handle_for_free_slot_returns_slot_free() {
    let mut table = EntitySlotTable::new();
    let entity = table.spawn().unwrap();
    table.despawn(entity).unwrap();

    let synthesized = Entity::new(entity.index(), entity.generation() + 1);

    assert_eq!(
        table.despawn(synthesized),
        Err(EntityError::SlotFree {
            index: entity.index(),
        })
    );
    assert!(!table.is_alive(synthesized));
}

#[test]
fn free_list_reuses_lowest_available_index_deterministically() {
    let mut table = EntitySlotTable::new();
    let first = table.spawn().unwrap();
    let second = table.spawn().unwrap();
    let third = table.spawn().unwrap();

    table.despawn(second).unwrap();
    let reused = table.spawn().unwrap();

    assert_eq!(first.index(), 0);
    assert_eq!(second.index(), 1);
    assert_eq!(third.index(), 2);
    assert_eq!(reused.index(), 1);
    assert_eq!(table.len(), 3);
}

#[test]
fn capacity_reflects_allocated_slot_storage_not_live_count() {
    let mut table = EntitySlotTable::with_capacity(4);

    assert!(table.capacity() >= 4);
    assert_eq!(table.len(), 0);

    let a = table.spawn().unwrap();
    let b = table.spawn().unwrap();
    table.despawn(a).unwrap();

    assert_eq!(table.len(), 1);
    assert!(table.capacity() >= 2);
    assert!(table.is_alive(b));
}

#[test]
fn high_index_handle_survives_lower_index_despawns() {
    let mut table = EntitySlotTable::new();
    let a = table.spawn().unwrap();
    let b = table.spawn().unwrap();
    let c = table.spawn().unwrap();
    let _d = table.spawn().unwrap();
    let e = table.spawn().unwrap();

    table.despawn(a).unwrap();
    table.despawn(b).unwrap();
    table.despawn(c).unwrap();

    assert_eq!(e.index(), 4);
    assert_eq!(table.len(), 2);
    assert!(table.is_alive(e));

    table.despawn(e).unwrap();
    assert!(!table.is_alive(e));
}
