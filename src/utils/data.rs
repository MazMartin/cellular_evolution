#[derive(Debug)]
pub struct IdxPair {
    pub a: usize,
    pub b: usize,
}

impl IdxPair {
    pub fn new(a: usize, b: usize) -> Self {
        Self { a, b }
    }

    // Distance between a and b
    pub fn span(&self) -> usize {
        self.b - self.a
    }
}

#[derive(Copy, Clone, Debug)]
enum HeapSlot<T> {
    None,        // free slot
    Allocated,   // reserved but uninitialized
    Some(T),     // initialized with value
}

pub struct Heap<T> {
    slots: Vec<HeapSlot<T>>,
}

impl<T: Clone> Heap<T> {
    // Create heap with given capacity, all slots free
    pub fn with_capacity(capacity: usize) -> Self {
        Heap {
            slots: vec![HeapSlot::None; capacity],
        }
    }
}

impl<T> Heap<T> {
    // Allocate contiguous free slots; return start index
    pub fn allocate_slots(&mut self, count: usize) -> usize {
        let mut i = 0;
        while i + count <= self.slots.len() {
            // Check if all slots in range are free
            if self.slots[i..i + count]
                .iter()
                .all(|slot| matches!(slot, HeapSlot::None))
            {
                // Mark slots as allocated
                for slot in &mut self.slots[i..i + count] {
                    *slot = HeapSlot::Allocated;
                }
                return i;
            }
            i += 1;
        }

        // No free block found, extend slots and allocate at end
        let start = self.slots.len();
        self.slots.extend((0..count).map(|_| HeapSlot::Allocated));
        start
    }

    // Free one slot at index
    pub fn free(&mut self, slot: usize) {
        self.slots[slot] = HeapSlot::None;
    }

    // Insert values into already allocated slots at start
    pub fn insert_vec(&mut self, start: usize, values: Vec<T>) {
        let end = start + values.len();
        assert!(end <= self.slots.len(), "Range out of bounds");
        assert!(
            self.slots[start..end]
                .iter()
                .all(|slot| matches!(slot, HeapSlot::Allocated)),
            "All target slots must be Allocated"
        );

        for (slot, value) in self.slots[start..end].iter_mut().zip(values) {
            *slot = HeapSlot::Some(value);
        }
    }

    // Allocate slots and insert values immediately
    pub fn insert_alloc_vec(&mut self, values: Vec<T>) {
        let start = self.allocate_slots(values.len());
        self.insert_vec(start, values);
    }

    // Get immutable reference to value at index
    pub fn get(&self, index: usize) -> &T {
        match self.slots.get(index) {
            Some(HeapSlot::Some(value)) => value,
            Some(HeapSlot::Allocated) => {
                panic!("Slot at index {index} is allocated but uninitialized")
            }
            Some(HeapSlot::None) => panic!("Slot at index {index} is free (None)"),
            None => panic!("Index {index} out of bounds"),
        }
    }

    // Get mutable reference to value at index
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        match self.slots.get_mut(index) {
            Some(HeapSlot::Some(value)) => value,
            Some(HeapSlot::Allocated) => {
                panic!("Slot at index {index} is allocated but uninitialized")
            }
            Some(HeapSlot::None) => panic!("Slot at index {index} is free (None)"),
            None => panic!("Index {index} out of bounds"),
        }
    }

    // Get mutable references to two distinct values safely
    pub fn get_mut_pair(&mut self, a: usize, b: usize) -> (&mut T, &mut T) {
        assert_ne!(a, b, "Indices must be different");

        if a < b {
            let (left, right) = self.slots.split_at_mut(b);
            let first = match &mut left[a] {
                HeapSlot::Some(v) => v,
                _ => panic!("Slot at index {} not initialized", a),
            };
            let second = match &mut right[0] {
                HeapSlot::Some(v) => v,
                _ => panic!("Slot at index {} not initialized", b),
            };
            (first, second)
        } else {
            let (left, right) = self.slots.split_at_mut(a);
            let second = match &mut left[b] {
                HeapSlot::Some(v) => v,
                _ => panic!("Slot at index {} not initialized", b),
            };
            let first = match &mut right[0] {
                HeapSlot::Some(v) => v,
                _ => panic!("Slot at index {} not initialized", a),
            };
            (first, second)
        }
    }

    // Iterator over all initialized values
    pub fn flatten_iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.slots.iter().filter_map(|slot| {
            if let HeapSlot::Some(value) = slot {
                Some(value)
            } else {
                None
            }
        })
    }

    // Mutable iterator over all initialized values
    pub fn flatten_iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.slots.iter_mut().filter_map(|slot| {
            if let HeapSlot::Some(value) = slot {
                Some(value)
            } else {
                None
            }
        })
    }

    // Iterator over (original_index, flattened_index, &value)
    pub fn flatten_enumerate(&self) -> impl Iterator<Item = (usize, usize, &T)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(original_index, cell)| match cell {
                HeapSlot::Some(value) => Some((original_index, value)),
                _ => None,
            })
            .enumerate()
            .map(|(flattened_index, (original_index, value))| {
                (original_index, flattened_index, value)
            })
    }
}
