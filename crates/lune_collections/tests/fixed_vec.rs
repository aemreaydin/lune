use lune_collections::{CollectionError, FixedVec};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn fixed_vec_starts_empty_with_const_capacity() {
    let values = FixedVec::<u32, 3>::new();

    assert_eq!(values.capacity(), 3);
    assert_eq!(values.len(), 0);
    assert!(values.is_empty());
    assert!(!values.is_full());
}

#[test]
fn fixed_vec_pushes_until_capacity_and_preserves_order() {
    let mut values = FixedVec::<u32, 3>::new();

    values.push(10).unwrap();
    values.push(20).unwrap();
    values.push(30).unwrap();

    assert_eq!(values.len(), 3);
    assert!(values.is_full());
    assert_eq!(values.as_slice(), &[10, 20, 30]);
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), vec![10, 20, 30]);
}

#[test]
fn fixed_vec_rejects_push_when_full_without_changing_contents() {
    let mut values = FixedVec::<u32, 2>::new();

    values.push(1).unwrap();
    values.push(2).unwrap();

    let err = values.push(3).unwrap_err();

    assert_eq!(err, CollectionError::FixedCapacityExceeded { capacity: 2 });
    assert_eq!(values.len(), 2);
    assert_eq!(values.as_slice(), &[1, 2]);
}

#[test]
fn fixed_vec_pop_returns_last_element_and_updates_len() {
    let mut values = FixedVec::<u32, 2>::new();

    values.push(1).unwrap();
    values.push(2).unwrap();

    assert_eq!(values.pop(), Some(2));
    assert_eq!(values.pop(), Some(1));
    assert_eq!(values.pop(), None);
    assert!(values.is_empty());
}

#[test]
fn fixed_vec_clear_drops_initialized_elements_and_allows_reuse() {
    let drops = Rc::new(Cell::new(0));
    let mut values = FixedVec::<DropCounter, 2>::new();

    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();

    values.clear();

    assert_eq!(drops.get(), 2);
    assert!(values.is_empty());

    values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    assert_eq!(values.len(), 1);
}

#[test]
fn fixed_vec_drop_only_drops_initialized_elements() {
    let drops = Rc::new(Cell::new(0));

    {
        let mut values = FixedVec::<DropCounter, 4>::new();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
        values.push(DropCounter::new(Rc::clone(&drops))).unwrap();
    }

    assert_eq!(drops.get(), 2);
}

#[test]
fn fixed_vec_mut_slice_allows_in_place_updates() {
    let mut values = FixedVec::<u32, 2>::new();

    values.push(1).unwrap();
    values.push(2).unwrap();
    values.as_mut_slice()[1] = 20;

    assert_eq!(values.as_slice(), &[1, 20]);
}

#[test]
fn fixed_vec_supports_zero_capacity() {
    let mut values = FixedVec::<u32, 0>::new();

    assert_eq!(values.capacity(), 0);
    assert!(values.is_empty());
    assert!(values.is_full());

    let err = values.push(1).unwrap_err();
    assert_eq!(err, CollectionError::FixedCapacityExceeded { capacity: 0 });
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
