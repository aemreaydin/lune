use lune_collections::{CollectionError, RingBuffer, RingBufferOverflowPolicy};
use proptest::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

#[test]
fn ring_buffer_starts_empty_with_explicit_policy() {
    let buffer = RingBuffer::<i32, 3>::new(RingBufferOverflowPolicy::Reject);

    assert_eq!(buffer.capacity(), 3);
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
    assert!(!buffer.is_full());
    assert_eq!(buffer.overflow_policy(), RingBufferOverflowPolicy::Reject);
    assert_eq!(buffer.as_slices(), (&[][..], &[][..]));
    assert_eq!(buffer.iter().len(), 0);
}

#[test]
fn rejecting_policy_reports_full_without_changing_contents() {
    let mut buffer = RingBuffer::<&str, 3>::rejecting();

    assert_eq!(buffer.push_back("a"), Ok(None));
    assert_eq!(buffer.push_back("b"), Ok(None));
    assert_eq!(buffer.push_back("c"), Ok(None));
    assert!(buffer.is_full());

    let error = buffer.push_back("d").unwrap_err();
    assert_eq!(error, CollectionError::RingBufferFull { capacity: 3 });
    assert_eq!(
        buffer.iter().copied().collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
}

#[test]
fn overwrite_policy_returns_oldest_value_and_keeps_latest_items() {
    let mut buffer = RingBuffer::<i32, 3>::overwriting_oldest();

    assert_eq!(buffer.push_back(1), Ok(None));
    assert_eq!(buffer.push_back(2), Ok(None));
    assert_eq!(buffer.push_back(3), Ok(None));
    assert_eq!(buffer.push_back(4), Ok(Some(1)));
    assert_eq!(buffer.push_back(5), Ok(Some(2)));

    assert_eq!(buffer.len(), 3);
    assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
}

#[test]
fn pop_front_returns_items_in_fifo_order_and_marks_empty() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();

    assert_eq!(buffer.pop_front(), None);
    buffer.push_back(10).unwrap();
    buffer.push_back(20).unwrap();
    buffer.push_back(30).unwrap();

    assert_eq!(buffer.pop_front(), Some(10));
    assert_eq!(buffer.pop_front(), Some(20));
    assert_eq!(buffer.pop_front(), Some(30));
    assert_eq!(buffer.pop_front(), None);
    assert!(buffer.is_empty());
    assert!(!buffer.is_full());
}

#[test]
fn wraparound_preserves_iteration_order() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();

    for value in 0..4 {
        buffer.push_back(value).unwrap();
    }

    assert_eq!(buffer.pop_front(), Some(0));
    assert_eq!(buffer.pop_front(), Some(1));
    buffer.push_back(4).unwrap();
    buffer.push_back(5).unwrap();

    assert!(buffer.is_full());
    assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4, 5]);
}

#[test]
fn as_slices_exposes_wrapped_storage_in_fifo_order() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();

    for value in 0..4 {
        buffer.push_back(value).unwrap();
    }

    assert_eq!(buffer.pop_front(), Some(0));
    assert_eq!(buffer.pop_front(), Some(1));
    buffer.push_back(4).unwrap();
    buffer.push_back(5).unwrap();

    let (first, second) = buffer.as_slices();
    assert_eq!(first, &[2, 3]);
    assert_eq!(second, &[4, 5]);
}

#[test]
fn clear_drops_initialized_elements_and_allows_reuse() {
    #[derive(Debug)]
    struct DropProbe(Rc<Cell<usize>>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let drops = Rc::new(Cell::new(0));

    {
        let mut buffer = RingBuffer::<DropProbe, 4>::rejecting();
        buffer.push_back(DropProbe(Rc::clone(&drops))).unwrap();
        buffer.push_back(DropProbe(Rc::clone(&drops))).unwrap();
        buffer.push_back(DropProbe(Rc::clone(&drops))).unwrap();
        assert_eq!(drops.get(), 0);

        buffer.clear();
        assert_eq!(drops.get(), 3);

        buffer.push_back(DropProbe(Rc::clone(&drops))).unwrap();
        assert_eq!(drops.get(), 3);
    }

    assert_eq!(drops.get(), 4);
}

#[test]
fn zero_capacity_rejecting_buffer_is_both_empty_and_full() {
    let mut buffer = RingBuffer::<i32, 0>::rejecting();

    assert_eq!(buffer.capacity(), 0);
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
    assert!(buffer.is_full());

    let error = buffer.push_back(1).unwrap_err();
    assert_eq!(error, CollectionError::RingBufferFull { capacity: 0 });
}

#[test]
fn zero_capacity_overwriting_buffer_rejects_push() {
    let mut buffer = RingBuffer::<i32, 0>::overwriting_oldest();

    assert_eq!(buffer.capacity(), 0);
    assert!(buffer.is_empty());
    assert!(buffer.is_full());

    let error = buffer.push_back(1).unwrap_err();
    assert_eq!(error, CollectionError::RingBufferFull { capacity: 0 });
}

#[test]
fn as_slices_returns_contiguous_first_slice_when_storage_does_not_wrap() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();

    buffer.push_back(1).unwrap();
    buffer.push_back(2).unwrap();
    buffer.push_back(3).unwrap();

    let (first, second) = buffer.as_slices();
    assert_eq!(first, &[1, 2, 3]);
    assert_eq!(second, &[][..]);
}

#[test]
fn into_iterator_for_ref_yields_fifo_order() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();
    buffer.push_back(1).unwrap();
    buffer.push_back(2).unwrap();
    buffer.push_back(3).unwrap();

    let mut collected = Vec::new();
    for value in &buffer {
        collected.push(*value);
    }
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
fn iter_exact_size_len_and_size_hint_track_remaining_across_next() {
    let mut buffer = RingBuffer::<i32, 4>::rejecting();
    buffer.push_back(1).unwrap();
    buffer.push_back(2).unwrap();
    buffer.push_back(3).unwrap();

    let mut iter = buffer.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));

    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.size_hint(), (2, Some(2)));

    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn clear_drops_correct_elements_after_wraparound() {
    struct IdProbe {
        id: u32,
        log: Rc<RefCell<Vec<u32>>>,
    }

    impl Drop for IdProbe {
        fn drop(&mut self) {
            self.log.borrow_mut().push(self.id);
        }
    }

    let log = Rc::new(RefCell::new(Vec::<u32>::new()));
    let make = |id| IdProbe {
        id,
        log: Rc::clone(&log),
    };

    let mut buffer = RingBuffer::<IdProbe, 4>::rejecting();
    buffer.push_back(make(1)).unwrap();
    buffer.push_back(make(2)).unwrap();
    buffer.push_back(make(3)).unwrap();
    buffer.push_back(make(4)).unwrap();

    drop(buffer.pop_front());
    drop(buffer.pop_front());
    buffer.push_back(make(5)).unwrap();

    assert_eq!(log.borrow().clone(), vec![1, 2]);

    buffer.clear();

    let mut after_clear = log.borrow().clone();
    after_clear.sort();
    assert_eq!(after_clear, vec![1, 2, 3, 4, 5]);
}

#[test]
fn drop_runs_destructors_for_correct_elements_after_wraparound() {
    struct IdProbe {
        id: u32,
        log: Rc<RefCell<Vec<u32>>>,
    }

    impl Drop for IdProbe {
        fn drop(&mut self) {
            self.log.borrow_mut().push(self.id);
        }
    }

    let log = Rc::new(RefCell::new(Vec::<u32>::new()));
    let make = |id| IdProbe {
        id,
        log: Rc::clone(&log),
    };

    {
        let mut buffer = RingBuffer::<IdProbe, 4>::rejecting();
        buffer.push_back(make(1)).unwrap();
        buffer.push_back(make(2)).unwrap();
        buffer.push_back(make(3)).unwrap();
        buffer.push_back(make(4)).unwrap();

        drop(buffer.pop_front());
        drop(buffer.pop_front());
        buffer.push_back(make(5)).unwrap();
    }

    let mut dropped = log.borrow().clone();
    dropped.sort();
    assert_eq!(dropped, vec![1, 2, 3, 4, 5]);
}

#[test]
fn ring_buffer_supports_zero_sized_types() {
    let mut buffer = RingBuffer::<(), 3>::rejecting();

    buffer.push_back(()).unwrap();
    buffer.push_back(()).unwrap();
    buffer.push_back(()).unwrap();
    assert!(buffer.is_full());
    assert_eq!(
        buffer.push_back(()),
        Err(CollectionError::RingBufferFull { capacity: 3 })
    );

    assert_eq!(buffer.pop_front(), Some(()));
    assert_eq!(buffer.len(), 2);

    let mut overwriting = RingBuffer::<(), 2>::overwriting_oldest();
    assert_eq!(overwriting.push_back(()), Ok(None));
    assert_eq!(overwriting.push_back(()), Ok(None));
    assert_eq!(overwriting.push_back(()), Ok(Some(())));
    assert_eq!(overwriting.iter().count(), 2);
}

#[derive(Debug, Clone)]
enum Operation {
    Push(u8),
    Pop,
    Clear,
}

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        4 => any::<u8>().prop_map(Operation::Push),
        2 => Just(Operation::Pop),
        1 => Just(Operation::Clear),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        cases: if cfg!(miri) { 2 } else { 256 },
        ..ProptestConfig::default()
    })]

    #[test]
    fn rejecting_ring_buffer_matches_bounded_vecdeque(
        operations in proptest::collection::vec(operation_strategy(), 0..256)
    ) {
        const CAP: usize = 4;
        let mut ring = RingBuffer::<u8, CAP>::rejecting();
        let mut model: VecDeque<u8> = VecDeque::new();

        for operation in operations {
            match operation {
                Operation::Push(value) => {
                    if model.len() == CAP {
                        prop_assert_eq!(
                            ring.push_back(value),
                            Err(CollectionError::RingBufferFull { capacity: CAP })
                        );
                    } else {
                        prop_assert_eq!(ring.push_back(value), Ok(None));
                        model.push_back(value);
                    }
                }
                Operation::Pop => {
                    prop_assert_eq!(ring.pop_front(), model.pop_front());
                }
                Operation::Clear => {
                    ring.clear();
                    model.clear();
                }
            }

            prop_assert_eq!(ring.len(), model.len());
            prop_assert_eq!(ring.is_empty(), model.is_empty());
            prop_assert_eq!(ring.is_full(), model.len() == CAP);
            let collected: Vec<u8> = ring.iter().copied().collect();
            let expected: Vec<u8> = model.iter().copied().collect();
            prop_assert_eq!(collected, expected);
        }
    }

    #[test]
    fn overwriting_ring_buffer_matches_bounded_vecdeque(
        operations in proptest::collection::vec(operation_strategy(), 0..256)
    ) {
        const CAP: usize = 4;
        let mut ring = RingBuffer::<u8, CAP>::overwriting_oldest();
        let mut model: VecDeque<u8> = VecDeque::new();

        for operation in operations {
            match operation {
                Operation::Push(value) => {
                    let displaced = if model.len() == CAP {
                        model.pop_front()
                    } else {
                        None
                    };
                    model.push_back(value);
                    prop_assert_eq!(ring.push_back(value), Ok(displaced));
                }
                Operation::Pop => {
                    prop_assert_eq!(ring.pop_front(), model.pop_front());
                }
                Operation::Clear => {
                    ring.clear();
                    model.clear();
                }
            }

            prop_assert_eq!(ring.len(), model.len());
            prop_assert_eq!(ring.is_empty(), model.is_empty());
            prop_assert_eq!(ring.is_full(), model.len() == CAP);
            let collected: Vec<u8> = ring.iter().copied().collect();
            let expected: Vec<u8> = model.iter().copied().collect();
            prop_assert_eq!(collected, expected);
        }
    }
}
