use lune_collections::SmallVec;
use proptest::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn small_vec_starts_empty_with_inline_capacity() {
    let values = SmallVec::<u32, 4>::new();

    assert_eq!(values.inline_capacity(), 4);
    assert_eq!(values.capacity(), 4);
    assert_eq!(values.len(), 0);
    assert!(values.is_empty());
    assert!(!values.is_spilled());
}

#[test]
fn small_vec_uses_inline_storage_until_capacity() {
    let mut values = SmallVec::<u32, 2>::new();

    values.push(10).unwrap();
    values.push(20).unwrap();

    assert_eq!(values.len(), 2);
    assert_eq!(values.capacity(), 2);
    assert!(!values.is_spilled());
    assert_eq!(values.as_slice(), &[10, 20]);
}

#[test]
fn small_vec_spills_after_inline_capacity_and_preserves_order() {
    let mut values = SmallVec::<u32, 2>::new();

    values.push(10).unwrap();
    values.push(20).unwrap();
    values.push(30).unwrap();

    assert_eq!(values.inline_capacity(), 2);
    assert!(values.capacity() >= 3);
    assert_eq!(values.len(), 3);
    assert!(values.is_spilled());
    assert_eq!(values.as_slice(), &[10, 20, 30]);
}

#[test]
fn small_vec_pop_crosses_spill_boundary_in_lifo_order() {
    let mut values = SmallVec::<u32, 2>::new();

    values.push(1).unwrap();
    values.push(2).unwrap();
    values.push(3).unwrap();

    assert_eq!(values.pop(), Some(3));
    assert_eq!(values.pop(), Some(2));
    assert_eq!(values.pop(), Some(1));
    assert_eq!(values.pop(), None);
    assert!(values.is_empty());
}

#[test]
fn small_vec_mut_slice_allows_in_place_updates_after_spill() {
    let mut values = SmallVec::<u32, 1>::new();

    values.push(1).unwrap();
    values.push(2).unwrap();
    values.as_mut_slice()[1] = 20;

    assert_eq!(values.as_slice(), &[1, 20]);
}

#[test]
fn small_vec_clear_drops_inline_and_spilled_elements_and_allows_reuse() {
    let drops = Rc::new(Cell::new(0));
    let mut values = SmallVec::<DropCounter, 2>::new();

    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();

    values.clear();

    assert_eq!(drops.get(), 3);
    assert!(values.is_empty());

    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    assert_eq!(values.len(), 1);
}

#[test]
fn small_vec_drop_only_drops_initialized_elements_once() {
    let drops = Rc::new(Cell::new(0));

    {
        let mut values = SmallVec::<DropCounter, 2>::new();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    }

    assert_eq!(drops.get(), 3);
}

#[test]
fn small_vec_drops_inline_only_elements_when_dropped() {
    let drops = Rc::new(Cell::new(0));

    {
        let mut values = SmallVec::<DropCounter, 4>::new();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
        assert!(!values.is_spilled());
    }

    assert_eq!(drops.get(), 2);
}

#[test]
fn moved_small_vec_preserves_initialized_elements() {
    let mut values = SmallVec::<u32, 2>::new();
    values.push(1).unwrap();
    values.push(2).unwrap();
    values.push(3).unwrap();

    let moved = values;

    assert_eq!(moved.as_slice(), &[1, 2, 3]);
}

#[test]
fn moved_inline_small_vec_preserves_initialized_elements() {
    let mut values = SmallVec::<u32, 4>::new();
    values.push(1).unwrap();
    values.push(2).unwrap();
    assert!(!values.is_spilled());

    let moved = values;

    assert!(!moved.is_spilled());
    assert_eq!(moved.as_slice(), &[1, 2]);
}

#[test]
fn small_vec_zero_inline_capacity_spills_on_first_push() {
    let mut values = SmallVec::<u32, 0>::new();

    assert_eq!(values.inline_capacity(), 0);
    assert!(!values.is_spilled());

    values.push(1).unwrap();

    assert!(values.is_spilled());
    assert_eq!(values.len(), 1);
    assert!(values.capacity() >= 1);
    assert_eq!(values.as_slice(), &[1]);
}

#[test]
fn small_vec_supports_zero_sized_types() {
    let mut values = SmallVec::<(), 2>::new();

    values.push(()).unwrap();
    values.push(()).unwrap();
    values.push(()).unwrap();

    assert_eq!(values.len(), 3);
    assert!(values.is_spilled());
    assert_eq!(values.as_slice(), &[(), (), ()]);

    assert_eq!(values.pop(), Some(()));
    assert_eq!(values.pop(), Some(()));
    assert_eq!(values.pop(), Some(()));
    assert_eq!(values.pop(), None);
}

#[test]
fn small_vec_remains_spilled_after_pop_below_inline_capacity() {
    let mut values = SmallVec::<u32, 2>::new();
    values.push(1).unwrap();
    values.push(2).unwrap();
    values.push(3).unwrap();
    assert!(values.is_spilled());

    values.pop();
    values.pop();

    assert_eq!(values.len(), 1);
    assert!(values.is_spilled());
}

#[test]
fn small_vec_remains_spilled_after_clear() {
    let mut values = SmallVec::<u32, 2>::new();
    values.push(1).unwrap();
    values.push(2).unwrap();
    values.push(3).unwrap();
    assert!(values.is_spilled());
    let spilled_capacity = values.capacity();

    values.clear();

    assert!(values.is_empty());
    assert!(values.is_spilled());
    assert_eq!(values.capacity(), spilled_capacity);
    assert!(values.capacity() > values.inline_capacity());
}

#[test]
fn small_vec_capacity_after_spill_exceeds_inline_and_holds_len() {
    let mut values = SmallVec::<u32, 2>::new();
    values.push(1).unwrap();
    values.push(2).unwrap();
    values.push(3).unwrap();

    assert!(values.is_spilled());
    assert!(values.capacity() > values.inline_capacity());
    assert!(values.capacity() >= values.len());
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        cases: if cfg!(miri) { 2 } else { 256 },
        .. ProptestConfig::default()
    })]

    #[test]
    fn small_vec_push_pop_matches_vec_for_operation_sequences(
        operations in proptest::collection::vec(operation_strategy(), 0..256)
    ) {
        let mut small = SmallVec::<u8, 4>::new();
        let mut standard = Vec::new();

        for operation in operations {
            match operation {
                Operation::Push(value) => {
                    small.push(value).unwrap();
                    standard.push(value);
                }
                Operation::Pop => {
                    prop_assert_eq!(small.pop(), standard.pop());
                }
            }

            prop_assert_eq!(small.as_slice(), standard.as_slice());
            prop_assert!(small.capacity() >= small.len());
        }
    }
}

#[derive(Clone, Debug)]
enum Operation {
    Push(u8),
    Pop,
}

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        3 => any::<u8>().prop_map(Operation::Push),
        1 => Just(Operation::Pop),
    ]
}

struct DropCounter {
    drops: Rc<Cell<usize>>,
}

impl DropCounter {
    fn new(drops: Rc<Cell<usize>>) -> Self {
        Self { drops }
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
