use std::cmp::Ordering;

fn children(parent: usize) -> (usize, usize) {
    let left = parent * 2 + 1;
    (left, left + 1)
}

fn parent(child: usize) -> Option<usize> {
    match child {
        0 => None,
        _ => Some((child - 1) / 2),
    }
}

/// Makes a max heap out of `data`, according to `compare`.
///
/// "Max" in this context is defined by the element `a` which when compared
/// with any other element `b` via `compare(a, b)` may return `Greater` or
/// `Equal`, but not `Less`.
/// 
/// # Examples
/// 
/// ```
/// # use stdex::algorithms::*;
/// let mut heap = [1, 3, 5, 7, 9, 0, 2, 4, 6, 8];
/// let compare = |a: &i32, b: &i32| a.cmp(b);
/// make_heap(&mut heap, compare);
/// assert!(is_heap(&heap, compare));
/// ```
/// 
/// ```
/// // make a min heap by reversing the comparison
/// # use stdex::algorithms::*;
/// let mut heap = [0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5];
/// let compare = |a: &i32, b: &i32| a.cmp(b).reverse();
/// make_heap(&mut heap, compare);
/// assert!(is_heap(&heap, compare));
/// assert_eq!(heap[0], -5);
/// ```
pub fn make_heap<T,F>(data: &mut [T], compare: F)
where F: Clone + FnMut(&T, &T) -> Ordering {
    let mut len = 1;
    while len < data.len() {
        len += 1;
        push_heap(&mut data[0..len], compare.clone());
    }
}

/// Assumes all but the last element of `data` is a max heap according to
/// `compare`, then pushes the last element onto it.
/// 
/// "Max" in this context is defined by the element `a` which when compared
/// with any other element `b` via `compare(a, b)` may return `Greater` or
/// `Equal`, but not `Less`.
/// 
/// # Examples
/// 
/// ```
/// # use stdex::algorithms::*;
/// # let mut rn = 0;
/// # let mut random_number_generator = || { rn += 1; rn };
/// # let compare = |a: &i32, b: &i32| a.cmp(b);
/// let mut heap = Vec::new();
/// for _ in 0..20 {
///     let x = random_number_generator();
///     heap.push(x);
///     push_heap(&mut heap, compare);
///     assert!(is_heap(&heap, compare));
/// }
/// ```
pub fn push_heap<T,F>(data: &mut [T], mut compare: F)
where F: FnMut(&T, &T) -> Ordering {
    let mut pos = data.len() - 1;
    while let Some(parent_pos) = parent(pos) {
        if compare(&data[pos], &data[parent_pos]) == Ordering::Greater {
            data.swap(pos, parent_pos);
            pos = parent_pos;
        } else {
            break;
        }
    }
}

/// Assumes `data` is a max heap according to `compare`, then moves the
/// max(i.e. the first) element to the back and restores `data` so that it is
/// a heap excluding the last element.
///
/// "Max" in this context is defined by the element `a` which when compared
/// with any other element `b` via `compare(a, b)` may return `Greater` or
/// `Equal`, but not `Less`.
/// 
/// # Examples
/// 
/// ```
/// # use stdex::algorithms::*;
/// # let gimme_some_random_ints = || vec![1,3,5,7,9,8,6,4,2,0];
/// # let compare = |a: &i32, b: &i32| a.cmp(b);
/// let mut heap: Vec<i32> = gimme_some_random_ints();
/// make_heap(&mut heap, compare);
/// while !heap.is_empty() {
///     pop_heap(&mut heap, compare);
///     heap.pop();
///     assert!(is_heap(&mut heap, compare));
/// }
/// ```
pub fn pop_heap<T,F>(data: &mut [T], mut compare: F)
where F: FnMut(&T, &T) -> Ordering {
    let last = data.len() - 1;
    data.swap(0, last);

    let mut pos = 0;
    loop {
        let (left, right) = children(pos);

        if left >= last { break; }

        let next = {
            if right >= last {
                left
            } else {
                match compare(&data[right], &data[left]) {
                    Ordering::Greater => right,
                    _ => left,
                }
            }
        };

        match compare(&data[next], &data[pos]) {
            Ordering::Greater => {
                data.swap(next, pos);
                pos = next;
            },
            _ => break,
        }
    }
}

/// Determines if `data` is a max heap according to `compare`.
/// 
/// "Max" in this context is defined by the element `a` which when compared
/// with any other element `b` via `compare(a, b)` may return `Greater` or
/// `Equal`, but not `Less`.
/// 
/// # Examples
/// 
/// ```
/// # use stdex::algorithms::*;
/// # let compare = |a: &i32, b: &i32| a.cmp(b);
/// let mut data = [1, 3, 5, 7, 9, 8, 6, 4, 2, 0];
/// assert!(!is_heap(&data, compare));
/// make_heap(&mut data, compare);
/// assert!(is_heap(&data, compare));
/// ```
pub fn is_heap<T,F>(data: &[T], compare: F) -> bool
where F: Clone + FnMut(&T, &T) -> Ordering {
    is_heap_from(data, 0, compare)
}

fn is_heap_from<T,F>(data: &[T], pos: usize, mut compare: F) -> bool
where F: Clone + FnMut(&T, &T) -> Ordering {
    let (left, right) = children(pos);

    if left >= data.len() { return true; }
    if compare(&data[left], &data[pos]) == Ordering::Greater {
        return false;
    }

    if right >= data.len() { return true; }
    if compare(&data[right], &data[pos]) == Ordering::Greater {
        return false;
    }

    is_heap_from(data, left, compare.clone())
    && is_heap_from(data, right, compare)
}

/// Sorts a heap.
/// 
/// Using the same function that was passed to create a heap with `make_heap`
/// or a series of calls to `push_heap`, sorts the heap array from `Less` to
/// `Greater`. Not to be confused with `heapsort`, which sorts an array in
/// arbitrary order using the Heap Sort algorithm.
pub fn sort_heap<T,F>(data: &mut [T], compare: F)
where F: Clone + FnMut(&T, &T) -> Ordering {
    let mut len = data.len();
    while len > 1 {
        pop_heap(&mut data[..len], compare.clone());
        len -= 1;
    }
}

/// Sort an array using the Heap Sort algorithm.
/// 
/// Not to be confused with `sort_heap`, which assumes the array is already
/// in heap order.
pub fn heapsort<T,F>(data: &mut [T], compare: F)
where F: Clone + FnMut(&T, &T) -> Ordering {
    make_heap(data, compare.clone());
    sort_heap(data, compare);
}

// shortcut functions for making and working min/max heaps

/// Can be passed as the comparison function for the heap functions in order
/// to work with a max heap. Equivalent to `|a, b| a.cmp(b)`.
pub fn max_heap_compare<T: Ord>(a: &T, b: &T) -> Ordering { a.cmp(b) }

/// Can be passed as the comparison function for the heap functions in order
/// to work with a min heap. Equivalent to `|a, b| b.cmp(a)`.
pub fn min_heap_compare<T: Ord>(a: &T, b: &T) -> Ordering { b.cmp(a) }

/// Equivalent to `make_heap(data, max_heap_compare)`
pub fn make_max_heap<T: Ord>(data: &mut [T]) {
    make_heap(data, max_heap_compare);
}

/// Equivalent to `push_heap(data, max_heap_compare)`
pub fn push_max_heap<T: Ord>(data: &mut [T]) {
    push_heap(data, max_heap_compare);
}

/// Equivalent to `pop_heap(data, max_heap_compare)`
pub fn pop_max_heap<T: Ord>(data: &mut [T]) {
    pop_heap(data, max_heap_compare);
}

/// Equivalent to `is_heap(data, max_heap_compare)`
pub fn is_max_heap<T: Ord>(data: &[T]) -> bool {
    is_heap(data, max_heap_compare)
}

/// Equivalent to `make_heap(data, min_heap_compare)`
pub fn make_min_heap<T: Ord>(data: &mut [T]) {
    make_heap(data, min_heap_compare);
}

/// Equivalent to `push_heap(data, min_heap_compare)`
pub fn push_min_heap<T: Ord>(data: &mut [T]) {
    push_heap(data, min_heap_compare);
}

/// Equivalent to `pop_heap(data, min_heap_compare)`
pub fn pop_min_heap<T: Ord>(data: &mut [T]) {
    pop_heap(data, min_heap_compare);
}

/// Equivalent to `is_heap(data, min_heap_compare)`
pub fn is_min_heap<T: Ord>(data: &[T]) -> bool {
    is_heap(data, min_heap_compare)
}

////////////////////////////////////////////////////////////////////////////////
// tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    #[test]
    fn test_push_heap() {
        let mut v: Vec<_> = random_integer_set(30);
        let compare = |a: &i32, b: &i32| a.cmp(b);
        assert!(!super::is_heap(&v, compare));
        let mut heap_len = 0;
        while heap_len < v.len() {
            heap_len += 1;
            let data = &mut v[..heap_len];
            super::push_heap(data, compare);
            assert!(super::is_heap(data, compare));
        }
    }

    #[test]
    fn test_pop_heap() {
        let mut v: Vec<_> = random_integer_set(30);
        let compare = |a: &i32, b: &i32| b.cmp(a);
        assert!(!super::is_heap(&v, compare));
        super::make_heap(&mut v, compare);
        assert!(super::is_heap(&v, compare));
        let mut heap_len = v.len();
        let mut old_top = v[0];
        while heap_len > 0 {
            assert_ne!(compare(&v[0], &old_top), Ordering::Greater);
            super::pop_heap(&mut v[..heap_len], compare);
            heap_len -= 1;
            assert!(super::is_heap(&v[..heap_len], compare));
            old_top = v[0];
        }
    }

    fn random_integer_set(count: usize) -> Vec<i32> {
        #[allow(overflowing_literals)]
        let integers = [
            0x91f63d52, 0xb90d30cd, 0x6f4d4f26, 0x77e426ba, 0x61f686d4, 0x302e8122, 0x06b22bc0, 0xb4d3e16a,
            0xee3c1e69, 0x98a9ab94, 0x6911b180, 0x312e857e, 0xf4f3435e, 0x0394e4b5, 0x2b4209a1, 0xc984e093,
            0x3ce9fb26, 0x492c827a, 0x18180ad5, 0xf576d934, 0x6a3c7615, 0x4ecb5996, 0x33f0c3e8, 0x7bab6230,
            0xef5d926f, 0xc485ac2f, 0xa518e3c8, 0x937abff5, 0xb2926e12, 0xbef3401b, 0xf3503774, 0x81fdc1cb,
            0x9f4250b4, 0xcbb6c7b8, 0xa42f1e35, 0x57bbf59a, 0x10cd9580, 0xc6390e4a, 0x2c4a3220, 0xc06b8563,
            0x5274ee0f, 0x1aad2f47, 0xccbe54ca, 0xd3500ff4, 0xeb65039c, 0xe06a611b, 0x17a0f09f, 0x32334acf,
            0x2171a66f, 0x8733064b, 0x13fc76fd, 0x0e7de6fb, 0x22635847, 0x323d4225, 0x5a108931, 0x0ab2b97d,
            0x9ccf077d, 0xb9aed901, 0x44833782, 0xd5933fdc, 0xfdfdef6b, 0x530241f6, 0x152e7834, 0xa21f148a,
            0x1ec5b67a, 0xa5c7c6bc, 0x00f47271, 0xa11fd1dc, 0xcf08982b, 0x77636515, 0xc95d9652, 0x6eee30c2,
            0x719564ca, 0x4de1d50c, 0x189e9fda, 0x39f0b8e9, 0xcf917e2b, 0xf69214c0, 0xfdf7d901, 0xe662a731,
            0x200945d3, 0x6e52f18e, 0xb98c684c, 0x81d72766, 0x054ca107, 0x7821b8c9, 0x6ef6f31e, 0x924cc9c8,
            0x49751615, 0x33c17496, 0xf0dce5fe, 0x0bd22941, 0x7d4807e4, 0x5d9ac853, 0x77679288, 0x9e44cbd3,
            0xad5dc98c, 0x69f2a5ce, 0xd5280ac4, 0x9dd55211, 0x722dddd4, 0x570cd54d, 0x90697b2c, 0x6059feac,
            0x306880f9, 0x410542a7, 0xfcca2303, 0x6c92ce2d, 0xf97495c0, 0x233ad5c5, 0x774be65b, 0x54ce1d83,
            0x56863754, 0x5a13ceeb, 0xb665dfa1, 0x6a6d2b6e, 0x36fd69b3, 0xb0c149dc, 0x021dc60f, 0x6d2fe86d,
            0x0ee033a9, 0xc62c2d12, 0x4ac612ee, 0x57c24d48, 0xa8018ae4, 0x4f079dc9, 0xf0007772, 0x584f6ebf,
            0xcced2af7, 0x8628daea, 0xdf36f2ea, 0xe3e0dbc2, 0x4983c97d, 0x2539b17b, 0xcbdca90a, 0x6302650e,
            0xb668394c, 0x3911bb51, 0xa16019c3, 0x9b34d264, 0x5465cf27, 0x3b46efed, 0x8d8e5219, 0x921c3558,
            0x8ea5cfa0, 0x8e31f8f5, 0x2b89511f, 0x9b255370, 0xf3161ca4, 0x8867cc0c, 0x02e5fe39, 0x29775af4,
            0xe513b968, 0x6f515218, 0x3d400384, 0x26ab1d69, 0x32cc9aa2, 0xd0f16296, 0x8fed55ae, 0x0b957e14,
            0x6f9ffc95, 0x2d07c541, 0xc9ab0d87, 0x355f5bc0, 0x0537157c, 0x593ad364, 0xfff99f6e, 0x04093d9d,
            0xd7d4e413, 0xb6b2f0d3, 0xe1a53ccd, 0xa4e15b6c, 0xc1e60d68, 0x09ab62ad, 0x9f44eeae, 0xc6303654,
            0x81b127f6, 0x9c1ab821, 0xecceb47d, 0x2d76c2d4, 0x84dc39cd, 0x95aa4274, 0x8aa5dfb9, 0xcf504c3e,
            0x3eb3ea00, 0x71490b2e, 0x020d6a32, 0xd3e25f8e, 0xfda84553, 0x5175e514, 0xa74c299f, 0x665ff9b4,
            0x8eb369bd, 0x082b592e, 0xa1495cf1, 0xd2a1d31a, 0x27e39edb, 0x9dd8499d, 0x4125ad3d, 0x1684dfdc,
            0xbb044fdf, 0x3c819822, 0xde13a0e3, 0x20afab06, 0xca5b96a1, 0xb517d1c2, 0xc974e581, 0x796806a9,
            0x7068d535, 0x09fdbbee, 0x5e5b0045, 0x2422d3cd, 0x275a8954, 0x7b2c4598, 0xba9465cf, 0xdb803858,
            0x0b04aaf5, 0x1b4d8a1d, 0xc474d174, 0x1c5dc88f, 0xe4fe3958, 0x343b09e6, 0x2f7d1750, 0xa4ffa018,
            0x00e0ab8b, 0xd09db2e2, 0xecc44e84, 0x6e7ba3e5, 0xd7c4ab35, 0x0e28a0a2, 0x727cb85c, 0xf48c4586,
            0x7e53b53a, 0xb88d11ce, 0x153b5336, 0x7722d97e, 0xb443ab5d, 0x86221ccb, 0x7c5934bb, 0x790cd9d2,
            0x59cd6dd0, 0x3454dda2, 0x8f5755ac, 0xa84c7c73, 0x4040a04e, 0xd5dd3cd7, 0xd316f452, 0x8963e415,
            0xffc0bcfa, 0x3bdcc913, 0xe8e25bba, 0xe3550c18, 0xc7090786, 0x7a87815e, 0xefd59c5e, 0xe2f7a710,
            0xf024a59e, 0xfc86a655, 0xdb1144f4, 0xae3aab49, 0x2b34c525, 0x1fc2722d, 0x8f7291e0, 0xd2c984f1,
            0x1e11b5e0, 0xfe0ab574, 0xcc8cabec, 0x4a35204d, 0x4a0090d8, 0x4b2ac827, 0x117b003b, 0xddf08571,
            0xcdae2b81, 0x06ca0181, 0x3ee1304c, 0x23241447, 0xe6487055, 0x2f293ad3, 0xa1fec13c, 0x85bd58b3,
            0x957f9384, 0xe37741d0, 0xc04fce8b, 0x2fd88733, 0x7177272c, 0x20e4f45a, 0xdcd8c5fa, 0x6b2b0b52,
            0x6ac68796, 0x919dd765, 0xfcdc32cd, 0x5770e926, 0x6b2d2a03, 0x28b152c8, 0xab881647, 0x627aa75c,
            0xb0be8b7e, 0x983bf7f1, 0x6348aa5b, 0x2423152b, 0x05ba3355, 0xa4e6d949, 0x86c0a1f8, 0x28ce287b,
            0x86630263, 0xf5fba219, 0x1cf361fe, 0x61152356, 0xf681efc7, 0x1ddaa329, 0x2a4a985e, 0x6f42d5a8,
            0xfec051ad, 0x5eba9217, 0xf84a6a6f, 0x810803dc, 0x56aaf0bf, 0x844e48d2, 0x2f6b470d, 0x1e8c9c95,
            0x041086ab, 0x31bee2cd, 0x9962a988, 0x3a1b962e, 0x79816d79, 0x76894d0b, 0x000d04d3, 0x8ec976a5,
            0x73c37ee8, 0xe1268bdc, 0x0be121ef, 0x951e7d6a, 0x1da96240, 0xb17c3a9b, 0xc5a0e07b, 0xc2e3051a,
            0x364d0e88, 0xa81c46f6, 0x617efd40, 0xf13895ae, 0xa39506ed, 0xc0fe1d7e, 0x5666b3c8, 0xfe7734ab,
            0xc67c9224, 0x08b6abf7, 0x254cbd8f, 0x0eb58082, 0x33ab47fb, 0x89e46a5e, 0xa319cb59, 0x5457350f,
            0xb74988b4, 0xfed45ce4, 0x2bd40603, 0x52796b5f, 0x3ddf2291, 0xb315ebad, 0xa53fbe4d, 0x6d278f84,
            0x3c774007, 0xcd21df0f, 0x28325dcf, 0x80c401ab, 0x72b15e99, 0x289bfe88, 0x6ec1700d, 0xb88e0295,
            0xa68f55a9, 0xa238d094, 0xf75e5b7a, 0x9197fe36, 0x90c072ad, 0x8530b21d, 0xc8323bf9, 0xa659aee8,
            0x1e302e2d, 0x4f7ad317, 0x795c7e68, 0x9ad8f0ac, 0xe043fd91, 0xbfe138c8, 0x12815ce4, 0x9c42ebcf,
            0x436c870c, 0x30634b4c, 0xe6ba3e21, 0xce6023a3, 0x4e3f5df4, 0x52f8af16, 0x1d64adde, 0x5de8c72c,
            0x9635b9e9, 0x4047598b, 0x7dcd9813, 0x8e134987, 0x4a85eda9, 0x71f9bcb5, 0xf34518a2, 0x9c4d534e,
            0x6df2a474, 0xb70a6a72, 0x3a872276, 0xa351738c, 0xa54fb224, 0x1a56370b, 0x827fcf2f, 0x8d09c4b5,
            0x2c1fc885, 0x4c11909b, 0x0049d323, 0xc52fb50a, 0x217a382a, 0xa8883451, 0x4bd04cb8, 0x3a359c58,
            0x8bae5c29, 0x7c2d84f8, 0x373bb0b0, 0xf1a4ef04, 0xd23c29dd, 0x92819a28, 0xeaed2d23, 0x5fc2a639,
            0x3ced6920, 0xcb604361, 0x8d07abcc, 0xf14aba4c, 0xbc4412f7, 0x091bcfe9, 0x20ae7e1d, 0xd93b61c4,
            0x4893cb38, 0xe7c087be, 0x38497425, 0x8b5e44c5, 0xa1f45fd9, 0x9604ec9c, 0x7683fcf9, 0x6486d9be,
            0x6df83325, 0x9764a6c3, 0xf4abdae8, 0x148f3fb9, 0xc6a04dd4, 0x0d631c52, 0x2ef8fa16, 0x8c234421,
            0xaf983135, 0xdf406d9b, 0x99dbadff, 0x9247abfe, 0x1ff86273, 0x36e10c73, 0x625238a2, 0x6c361fd4,
            0xec5c12c1, 0xf3247c9b, 0x157c1db4, 0xdd8270a2, 0xe6f604e5, 0x07fd787f, 0xd0f173fc, 0x74287138,
            0xcd890328, 0x8895ebf0, 0x6fe7e9fe, 0x4bd0c5ec, 0xec8e94f8, 0x4e61e62b, 0x17d4f80d, 0x2e033333,
            0xe2b08db5, 0x467d49b3, 0xffb7c52b, 0xfa9a9478, 0xb73c61b9, 0xc2141ac4, 0xd37a91a7, 0x01c61e64,
            0xb51888fa, 0x29e73681, 0xb98127f4, 0x28d8e079, 0xa87ec5de, 0x324339f4, 0xdd77a416, 0x1e08d849,
            0x5672a35a, 0xb2fd5230, 0x75fc086d, 0x99c2f9fb, 0xc97bdaa5, 0x2229d2bc, 0x02e56553, 0xa1de47fb,
            0x26494ac1, 0x383f333e, 0x347d13c7, 0x36ffe7b8, 0xc42caf57, 0xc1c112fa, 0x16522da4, 0x7984e68a,
            0xd3f0e4d9, 0x986d37ec, 0xdf157949, 0x2070ef0f, 0xa9c82af9, 0x5ba1000c, 0xa96137d4, 0xb4aecc65,
        ];

        integers[..count].into()
    }
}