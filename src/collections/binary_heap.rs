use std::cmp::Ordering;

use algorithms::*;

/// A binary heap ordered by a custom comparison function.
/// 
/// The comparison function should implement `FnMut(&T, &T) -> Ordering`,
/// where the argument which should be placed nearer the top of the heap
/// should return `Greater` or `Equal` when passed as the first argument.
#[derive(Clone)]
pub struct BinaryHeap<T, C> {
    data: Vec<T>,
    compare: C,
}

/// Don't use this.
/// 
/// This trait is used by BinaryHeap internally for its comparisons, as
/// opposed to directly using `FnMut(&T, &T) -> Ordering`. Please just
/// ignore it. It will disappear once `FnMut` implementation is stable.
pub trait Compare<T> {
    fn compare(&mut self, a: &T, b: &T) -> Ordering;
}

impl<T,C> BinaryHeap<T,C> {
    /// Create an empty heap.
    /// 
    /// No ordering will be selected by default, it must be specified in type
    /// annotations. If you just want a max heap or a min heap, you can use
    /// `MaxCompare` or `MinCompare` respectively.
    pub fn new() -> BinaryHeap<T,C>
    where C: Default {
        BinaryHeap {
            data: Vec::new(),
            compare: Default::default(),
        }
    }

    /// Returns the number of items in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    // Push an item onth the heap.
    pub fn push(&mut self, value: T)
    where C: Compare<T> {
        self.data.push(value);
        unsafe {
            let data = duplicate_mut_ref(&mut self.data);
            push_heap(data, |a, b| self.compare.compare(a, b));
        }
    }

    /// Removes the top item from the heap and returns it, or `None` if it is
    /// empty.
    pub fn pop(&mut self) -> Option<T>
    where C: Compare<T> {
        if self.data.len() > 0 {
            unsafe {
                let data = duplicate_mut_ref(&mut self.data);
                pop_heap(data,  |a, b| self.compare.compare(a, b));
            }
        }
        self.data.pop()
    }

    /// Returns the greatest item in the binary heap, or `None` if it is empty.
    pub fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    /// Returns a mutable reference to the greatest item in the binary heap,
    /// or `None` if it is empty.
    /// 
    /// # Safety
    /// Unsafe because modifying an element of the heap could potentially
    /// invalidate it.
    pub unsafe fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut(0)
    }

    /// Returns the heap's underlying Vec.
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Returns a mutable reference to the underlying Vec.
    /// 
    /// # Safety
    /// Unsafe because modifying an element of the heap could potentially
    /// invalidate it.
    pub unsafe fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }
}

/// Don't use this.
/// 
/// Like the Compare trait, you should not use this. It will disappear once
/// `FnMut` implementation is stable.
pub struct FnCompare<F>(pub F);
impl<T, F> Compare<T> for FnCompare<F>
where F: FnMut(&T, &T) -> Ordering {
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        self.0(a, b)
    }
}

impl<T,C> BinaryHeap<T, FnCompare<C>> {
    /// Constructs a binary heap that uses `compare` as its comparison function.
    /// 
    /// `compare(a, b)` should `Greater` if `a` should be placed above `b` in
    /// the heap, `Less` if `b` should be above `a`, and `Equal` if they should
    /// be considered equivalent.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use stdex::collections::binary_heap::BinaryHeap;
    /// // create a heap of tuples that is ordered by the
    /// // second element, while the first is ignored
    /// let heap: BinaryHeap<(f32, i32), _> = BinaryHeap::with_compare(
    ///     |a: &(f32, i32), b: &(f32, i32)| a.1.cmp(&b.1)
    /// );
    /// ```
    pub fn with_compare(compare: C) -> BinaryHeap<T,FnCompare<C>>
    where C: FnMut(&T, &T) -> Ordering {
        BinaryHeap {
            data: Vec::new(),
            compare: FnCompare(compare),
        }
    }
}

/// The comparison type for creating a max heap
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxCompare;
impl<T> Compare<T> for MaxCompare
where T: Ord {
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        max_heap_compare(a, b)
    }
}

impl<T> BinaryHeap<T, MaxCompare> {
    pub fn max_heap() -> BinaryHeap<T, MaxCompare> {
        BinaryHeap {
            data: Vec::new(),
            compare: Default::default(),
        }
    }
}

/// The comparison type for creating a min heap.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinCompare;
impl<T> Compare<T> for MinCompare
where T: Ord {
    fn compare(&mut self, a: &T, b: &T) -> Ordering {
        min_heap_compare(a, b)
    }
}

impl<T> BinaryHeap<T, MinCompare> {
    pub fn min_heap() -> BinaryHeap<T, MinCompare> {
        BinaryHeap {
            data: Vec::new(),
            compare: Default::default(),
        }
    }
}

unsafe fn duplicate_mut_ref<'a, 'b, T>(item: &'a mut T) -> &'b mut T {
    &mut *(item as *mut T)
}