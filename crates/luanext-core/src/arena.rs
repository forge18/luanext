//! Arena allocator for AST nodes.
//!
//! This module provides a bump allocator wrapper optimized for compiler
//! workloads. All AST nodes will be allocated from arenas, enabling:
//!
//! - O(1) bulk deallocation
//! - Better cache locality
//! - Faster allocation (bump pointer vs malloc)
//!
//! # Usage
//!
//! ```
//! use typedlua_core::arena::Arena;
//!
//! let arena = Arena::new();
//! let value: &i32 = arena.alloc(42);
//! assert_eq!(*value, 42);
//! ```
//!
//! # Design
//!
//! The `Arena` wraps `bumpalo::Bump` with compiler-specific helpers:
//!
//! - Allocation counting for metrics/debugging
//! - Convenient slice allocation methods
//! - String interning support
//!
//! Each compilation unit should have its own arena. The arena is not
//! thread-safe by design - parallel compilation uses separate arenas.

use bumpalo::Bump;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cell::Cell;

/// Arena allocator for AST nodes.
///
/// Wraps `bumpalo::Bump` with compiler-specific helpers and metrics.
/// All allocations are valid for the arena's lifetime.
///
/// # Example
///
/// ```
/// use typedlua_core::arena::Arena;
///
/// let arena = Arena::new();
///
/// // Allocate a single value
/// let num: &i32 = arena.alloc(42);
///
/// // Allocate a slice
/// let slice: &[i32] = arena.alloc_slice_copy(&[1, 2, 3]);
///
/// // Check metrics
/// println!("Allocations: {}", arena.allocation_count());
/// println!("Bytes used: {}", arena.allocated_bytes());
/// ```
pub struct Arena {
    bump: Bump,
    /// Track allocation count for debugging/metrics
    allocation_count: Cell<usize>,
}

impl Arena {
    /// Create a new arena with default capacity.
    ///
    /// The arena starts with a small initial chunk and grows as needed.
    #[inline]
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            allocation_count: Cell::new(0),
        }
    }

    /// Create a new arena with specified capacity hint.
    ///
    /// Use this when you know approximately how much memory will be needed
    /// (e.g., based on source file size). A good heuristic is 10-20x the
    /// source file size for AST allocation.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// // Pre-allocate for a ~50KB source file
    /// let arena = Arena::with_capacity(1024 * 1024); // 1MB
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
            allocation_count: Cell::new(0),
        }
    }

    /// Allocate a single value in the arena.
    ///
    /// Returns a reference valid for the arena's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let value: &String = arena.alloc(String::from("hello"));
    /// assert_eq!(value, "hello");
    /// ```
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &T {
        self.allocation_count.set(self.allocation_count.get() + 1);
        self.bump.alloc(value)
    }

    /// Allocate a slice by copying from an existing slice.
    ///
    /// This is the most efficient slice allocation method for `Copy` types.
    /// Useful for converting `Vec<T>` to arena-allocated slices.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let original = vec![1, 2, 3, 4, 5];
    /// let slice: &[i32] = arena.alloc_slice_copy(&original);
    /// assert_eq!(slice, &[1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &[T] {
        if slice.is_empty() {
            return &[];
        }
        self.bump.alloc_slice_copy(slice)
    }

    /// Allocate a slice by cloning from an existing slice.
    ///
    /// For non-Copy types that implement Clone. Slightly slower than
    /// `alloc_slice_copy` due to clone overhead.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let original = vec![String::from("a"), String::from("b")];
    /// let slice: &[String] = arena.alloc_slice_clone(&original);
    /// assert_eq!(slice[0], "a");
    /// ```
    #[inline]
    pub fn alloc_slice_clone<T: Clone>(&self, slice: &[T]) -> &[T] {
        if slice.is_empty() {
            return &[];
        }
        self.bump.alloc_slice_clone(slice)
    }

    /// Allocate a slice from an iterator.
    ///
    /// The iterator must be an `ExactSizeIterator` so the arena can
    /// pre-allocate the correct amount of memory.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let slice: &[i32] = arena.alloc_slice_fill_iter(0..5);
    /// assert_eq!(slice, &[0, 1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &[T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.bump.alloc_slice_fill_iter(iter)
    }

    /// Allocate a string slice in the arena.
    ///
    /// Useful for interned strings that need arena lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use typedlua_core::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let s: &str = arena.alloc_str("hello world");
    /// assert_eq!(s, "hello world");
    /// ```
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Get the number of allocations made from this arena.
    ///
    /// Useful for debugging and metrics. Note this counts calls to
    /// `alloc()`, not slice allocations.
    #[inline]
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.get()
    }

    /// Get the total bytes allocated from this arena.
    ///
    /// Includes all chunks allocated by the underlying bump allocator.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Reset the arena, deallocating all memory.
    ///
    /// This is O(1) - one of the key benefits of arena allocation.
    /// All references into the arena become invalid after this call.
    ///
    /// # Safety
    ///
    /// This method is safe because it takes `&mut self`, which means
    /// the borrow checker ensures no references to arena memory exist.
    #[inline]
    pub fn reset(&mut self) {
        self.bump.reset();
        self.allocation_count.set(0);
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arena")
            .field("allocations", &self.allocation_count.get())
            .field("bytes", &self.bump.allocated_bytes())
            .finish()
    }
}

//
// Arena Pooling
//

/// Global pool of reusable arenas for long-lived processes.
///
/// In watch mode and LSP scenarios, arenas are reused across many
/// compilation runs instead of allocating fresh arenas each time.
/// This reduces allocation overhead in transpiler workflows where
/// the same files are parsed hundreds of times.
static ARENA_POOL: Lazy<Mutex<Vec<Bump>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Maximum number of arenas to keep in the pool.
///
/// Limits memory usage - old arenas are dropped when pool exceeds this size.
/// Value is chosen to handle typical Rayon thread pool size (num_cpus).
const MAX_POOL_SIZE: usize = 16;

/// Execute a function with a pooled arena.
///
/// The arena is checked out from a global pool, reset to clear previous
/// allocations, and automatically returned to the pool when done.
///
/// This is the recommended way to parse files in long-lived processes
/// (LSP servers, watch mode, build servers). For one-shot compilation,
/// creating a fresh `Bump` directly may be simpler.
///
/// # Example
///
/// ```
/// use typedlua_core::arena::with_pooled_arena;
///
/// let result = with_pooled_arena(|arena| {
///     let value = arena.alloc(42);
///     *value
/// });
/// ```
///
/// # Thread Safety
///
/// This function is thread-safe and works correctly with parallel
/// parsing (e.g., Rayon). The pool is protected by a lightweight mutex.
pub fn with_pooled_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    // Try to get arena from pool (lock is held briefly)
    let mut arena = ARENA_POOL.lock().pop().unwrap_or_default();

    // Reset arena to clear previous allocations
    // SAFETY: We own the arena, so no references from previous uses exist
    arena.reset();

    // Execute user code
    let result = f(&arena);

    // Return arena to pool if under capacity
    let mut pool = ARENA_POOL.lock();
    if pool.len() < MAX_POOL_SIZE {
        pool.push(arena);
    }
    // else: arena dropped here (pool is full)

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc_single() {
        let arena = Arena::new();
        let value: &i32 = arena.alloc(42);
        assert_eq!(*value, 42);
        assert_eq!(arena.allocation_count(), 1);
    }

    #[test]
    fn test_arena_alloc_multiple() {
        let arena = Arena::new();
        let a: &i32 = arena.alloc(1);
        let b: &i32 = arena.alloc(2);
        let c: &i32 = arena.alloc(3);

        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
        assert_eq!(arena.allocation_count(), 3);
    }

    #[test]
    fn test_arena_alloc_slice_copy() {
        let arena = Arena::new();
        let original = vec![1, 2, 3, 4, 5];
        let slice: &[i32] = arena.alloc_slice_copy(&original);

        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_alloc_slice_clone() {
        let arena = Arena::new();
        let original = vec![String::from("a"), String::from("b")];
        let slice: &[String] = arena.alloc_slice_clone(&original);

        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0], "a");
        assert_eq!(slice[1], "b");
    }

    #[test]
    fn test_arena_alloc_slice_fill_iter() {
        let arena = Arena::new();
        let slice: &[i32] = arena.alloc_slice_fill_iter(0..5);

        assert_eq!(slice, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_arena_alloc_str() {
        let arena = Arena::new();
        let s: &str = arena.alloc_str("hello world");

        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_arena_alloc_empty_slice() {
        let arena = Arena::new();
        let empty: &[i32] = arena.alloc_slice_copy(&[]);

        assert!(empty.is_empty());
    }

    #[test]
    fn test_arena_with_capacity() {
        let arena = Arena::with_capacity(1024);
        let _: &i32 = arena.alloc(42);

        // Should have pre-allocated at least 1024 bytes
        assert!(arena.allocated_bytes() >= 1024);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new();
        let _: &i32 = arena.alloc(1);
        let _: &i32 = arena.alloc(2);
        let _: &i32 = arena.alloc(3);

        assert_eq!(arena.allocation_count(), 3);
        assert!(arena.allocated_bytes() > 0);

        arena.reset();

        assert_eq!(arena.allocation_count(), 0);
        // Note: allocated_bytes may still report memory due to chunk retention
    }

    #[test]
    fn test_arena_complex_struct() {
        #[derive(Debug, PartialEq)]
        struct ComplexStruct {
            name: String,
            values: Vec<i32>,
        }

        let arena = Arena::new();
        let obj = arena.alloc(ComplexStruct {
            name: String::from("test"),
            values: vec![1, 2, 3],
        });

        assert_eq!(obj.name, "test");
        assert_eq!(obj.values, vec![1, 2, 3]);
    }

    #[test]
    fn test_arena_nested_allocation() {
        let arena = Arena::new();

        // Simulate nested AST-like allocation
        let inner: &i32 = arena.alloc(42);
        let outer: &&i32 = arena.alloc(inner);

        assert_eq!(**outer, 42);
    }

    #[test]
    fn test_arena_debug() {
        let arena = Arena::new();
        let _: &i32 = arena.alloc(1);

        let debug = format!("{:?}", arena);
        assert!(debug.contains("Arena"));
        assert!(debug.contains("allocations"));
    }

    #[test]
    fn test_arena_default() {
        let arena = Arena::default();
        assert_eq!(arena.allocation_count(), 0);
    }

    /// Simulate AST-like nested structure allocation
    #[test]
    fn test_arena_ast_simulation() {
        // Simulate Expression-like enum
        #[derive(Debug)]
        enum Expr<'a> {
            Number(i64),
            Binary {
                left: &'a Expr<'a>,
                op: &'static str,
                right: &'a Expr<'a>,
            },
        }

        let arena = Arena::new();

        // Build: (1 + 2) * 3
        let one = arena.alloc(Expr::Number(1));
        let two = arena.alloc(Expr::Number(2));
        let add = arena.alloc(Expr::Binary {
            left: one,
            op: "+",
            right: two,
        });
        let three = arena.alloc(Expr::Number(3));
        let mul = arena.alloc(Expr::Binary {
            left: add,
            op: "*",
            right: three,
        });

        // Verify structure
        match mul {
            Expr::Binary { left, op, right } => {
                assert_eq!(*op, "*");
                match left {
                    Expr::Binary { op, .. } => assert_eq!(*op, "+"),
                    _ => panic!("Expected Binary"),
                }
                match right {
                    Expr::Number(n) => assert_eq!(*n, 3),
                    _ => panic!("Expected Number"),
                }
            }
            _ => panic!("Expected Binary"),
        }

        assert_eq!(arena.allocation_count(), 5);
    }

    /// Test allocation of statement-like vector
    #[test]
    fn test_arena_statement_list() {
        #[derive(Debug, Clone, PartialEq)]
        struct Statement {
            kind: &'static str,
            line: u32,
        }

        let arena = Arena::new();

        let stmts = vec![
            Statement {
                kind: "let",
                line: 1,
            },
            Statement {
                kind: "return",
                line: 2,
            },
            Statement {
                kind: "if",
                line: 3,
            },
        ];

        let allocated: &[Statement] = arena.alloc_slice_clone(&stmts);

        assert_eq!(allocated.len(), 3);
        assert_eq!(allocated[0].kind, "let");
        assert_eq!(allocated[1].kind, "return");
        assert_eq!(allocated[2].kind, "if");
    }

    /// Test large allocation to verify chunk handling
    #[test]
    fn test_arena_large_allocation() {
        let arena = Arena::new();

        // Allocate many items to trigger multiple chunks
        for i in 0..10_000 {
            let _: &i32 = arena.alloc(i);
        }

        assert_eq!(arena.allocation_count(), 10_000);
        assert!(arena.allocated_bytes() > 40_000); // At least 4 bytes per i32
    }

    //
    // Arena Pool Tests
    //

    /// Test that arenas are reused from the pool
    #[test]
    fn test_arena_pool_reuse() {
        // First allocation creates arena
        let addr1 = with_pooled_arena(|arena| arena as *const Bump);

        // Second allocation should reuse same arena (same address)
        let addr2 = with_pooled_arena(|arena| arena as *const Bump);

        assert_eq!(addr1, addr2, "Arena should be reused from pool");
    }

    /// Test that pool doesn't exceed MAX_POOL_SIZE
    #[test]
    fn test_arena_pool_capacity_limit() {
        // Fill pool beyond MAX_POOL_SIZE
        for _ in 0..(MAX_POOL_SIZE + 10) {
            with_pooled_arena(|arena| {
                arena.alloc(42);
            });
        }

        // Pool should not exceed MAX_POOL_SIZE
        let pool_size = ARENA_POOL.lock().len();
        assert!(
            pool_size <= MAX_POOL_SIZE,
            "Pool size {} exceeds MAX_POOL_SIZE {}",
            pool_size,
            MAX_POOL_SIZE
        );
    }

    /// Test that arenas are reset between uses
    #[test]
    fn test_arena_pool_reset() {
        with_pooled_arena(|arena| {
            // Allocate lots of data
            for i in 0..1000 {
                arena.alloc(i);
            }
        });

        with_pooled_arena(|arena| {
            // Arena should be reset (low allocation count)
            arena.alloc(1);
            let after = arena.allocated_bytes();

            // After reset, allocation should be small
            // Note: allocated_bytes() includes chunk overhead, so we just check
            // that it's reasonable (not thousands of bytes)
            assert!(
                after < 10000,
                "Arena not properly reset - allocated {} bytes",
                after
            );
        });
    }

    /// Test concurrent access to the arena pool
    #[test]
    fn test_arena_pool_concurrent() {
        use std::thread;

        let handles: Vec<_> = (0..8)
            .map(|i| {
                thread::spawn(move || {
                    with_pooled_arena(|arena| {
                        // Don't return arena reference - just use it
                        let val = arena.alloc(i);
                        *val
                    })
                })
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result < 8);
        }
    }

    /// Test that pool works correctly with actual AST-like allocations
    #[test]
    fn test_arena_pool_ast_simulation() {
        #[derive(Debug)]
        enum Expr<'a> {
            Number(i64),
            Binary {
                left: &'a Expr<'a>,
                op: &'static str,
                right: &'a Expr<'a>,
            },
        }

        // First parse
        let result1 = with_pooled_arena(|arena| {
            let one = arena.alloc(Expr::Number(1));
            let two = arena.alloc(Expr::Number(2));
            let add = arena.alloc(Expr::Binary {
                left: one,
                op: "+",
                right: two,
            });
            // Return something that doesn't reference the arena
            match add {
                Expr::Binary { op, .. } => *op,
                _ => panic!(),
            }
        });

        // Second parse with same arena (should be reset)
        let result2 = with_pooled_arena(|arena| {
            let three = arena.alloc(Expr::Number(3));
            let four = arena.alloc(Expr::Number(4));
            let mul = arena.alloc(Expr::Binary {
                left: three,
                op: "*",
                right: four,
            });
            match mul {
                Expr::Binary { op, .. } => *op,
                _ => panic!(),
            }
        });

        assert_eq!(result1, "+");
        assert_eq!(result2, "*");
    }
}
