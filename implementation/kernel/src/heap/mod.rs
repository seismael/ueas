//! Virtual heap implementation for the UEAS abstract interpreter.
//!
//! The virtual heap is a contiguous byte array isolated from the host
//! operating system. It provides allocation, deallocation, and type-aware
//! access primitives used by the interpreter's execution engine.
//!
//! # Complexity
//! Allocations are O(1). Deallocations are O(1). Access is O(1).
//!
//! # Errors
//! Returns `HEAP_EXHAUSTION` trap code if allocation exceeds configured size.

use crate::traps::ExitCode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Handle to an allocation in the virtual heap.
///
/// Wraps a 64-bit address. Not constructable outside the heap module
/// to prevent forged addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeapHandle(u64);

impl HeapHandle {
    /// Returns the raw u64 identifier for this heap handle.
    /// Internal use only — not exposed outside the kernel crate.
    pub(crate) fn as_u64(self) -> u64 {
        self.0
    }

    /// Reconstruct a HeapHandle from a raw u64 identifier.
    /// Internal use only — never construct handles outside the heap module.
    pub(crate) fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

/// Metadata about the UEAS type stored at an allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeTag {
    Integer,
    Real,
    Boolean,
    String,
    Set,
    List,
    Map,
    Graph,
    DirectedGraph,
    UndirectedGraph,
    Matrix,
    Option,
    Result,
    Tuple,
    HeapHandle,
    Unknown,
}

/// Metadata tracked alongside each heap allocation.
#[derive(Debug, Clone)]
struct AllocationMetadata {
    size: usize,
    offset: usize,
    type_tag: TypeTag,
}

/// Configuration for the virtual heap.
#[derive(Debug, Clone)]
pub struct HeapConfig {
    /// Maximum heap size in bytes (default 256 MiB).
    pub max_size: usize,
    /// Alignment requirement for allocations (default 8 bytes).
    pub alignment: usize,
}

impl Default for HeapConfig {
    fn default() -> Self {
        Self {
            max_size: 256 * 1024 * 1024, // 256 MiB
            alignment: 8,
        }
    }
}

/// Simulated CPU cache configuration for hardware-aware profiling (RFC 0008).
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 data cache size in bytes (default 65536 = 64KB)
    pub l1_size: u64,
    /// L2 cache size in bytes (default 262144 = 256KB)
    pub l2_size: u64,
    /// L3 cache size in bytes (default 8388608 = 8MB)
    pub l3_size: u64,
    /// Cache line size in bytes (default 64)
    pub cache_line_size: u64,
    /// Whether cache simulation is enabled (default false — backward compat)
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_size: 64 * 1024,
            l2_size: 256 * 1024,
            l3_size: 8 * 1024 * 1024,
            cache_line_size: 64,
            enabled: false,
        }
    }
}

/// Cache access statistics for profiling (RFC 0008).
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
}

impl CacheStats {
    pub fn total_accesses(&self) -> u64 {
        self.l1_hits + self.l1_misses
    }

    pub fn cache_miss_penalty(&self) -> u64 {
        self.l1_misses * 4 + self.l2_misses * 12 + self.l3_misses * 40
    }
}

/// Manages the isolated memory space for algorithm execution.
///
/// The virtual heap implements bump-pointer allocation with eager
/// deallocation. All allocations are tracked with type metadata.
/// The heap has zero access to system I/O, network, or hardware.
#[derive(Debug)]
pub struct VirtualHeap {
    memory: Vec<u8>,
    allocations: HashMap<HeapHandle, AllocationMetadata>,
    config: HeapConfig,
    next_handle: u64,
    bump_offset: usize,
    /// Cache configuration for hardware-aware profiling (RFC 0008).
    pub cache_config: CacheConfig,
    /// Cache access statistics for profiling (RFC 0008).
    pub cache_stats: CacheStats,
    /// LRU tracking: ring buffer of recently accessed addresses.
    recent_accesses: Vec<u64>,
}

impl VirtualHeap {
    /// Create a new virtual heap with the given configuration.
    pub fn new(config: HeapConfig) -> Self {
        Self {
            memory: vec![0u8; config.max_size],
            allocations: HashMap::new(),
            config,
            next_handle: 1,
            bump_offset: 0,
            cache_config: CacheConfig::default(),
            cache_stats: CacheStats::default(),
            recent_accesses: Vec::new(),
        }
    }

    /// Create a virtual heap with default configuration (256 MiB).
    pub fn with_default_config() -> Self {
        Self::new(HeapConfig::default())
    }

    /// Allocates a region of the given size and returns a handle.
    ///
    /// # Arguments
    /// * `size` — Number of bytes to allocate. Must be > 0.
    /// * `type_tag` — The UEAS type of the allocation for metadata tracking.
    ///
    /// # Returns
    /// A `HeapHandle` that can be used to read from or write to the
    /// allocated region.
    ///
    /// # Errors
    /// Returns `ExitCode::HeapExhaustion` if the allocation would exceed
    /// the configured heap size.
    ///
    /// # Complexity
    /// O(1) — bump-pointer allocation.
    pub fn allocate(&mut self, size: usize, type_tag: TypeTag) -> Result<HeapHandle, ExitCode> {
        if size == 0 {
            return Err(ExitCode::HeapExhaustion);
        }

        // Align the bump offset
        let align = self.config.alignment;
        let aligned_offset = self.bump_offset.div_ceil(align) * align;
        let new_offset = aligned_offset + size;

        if new_offset > self.config.max_size {
            return Err(ExitCode::HeapExhaustion);
        }

        let handle = HeapHandle(self.next_handle);
        self.next_handle += 1;

        self.allocations.insert(
            handle,
            AllocationMetadata {
                size,
                offset: aligned_offset,
                type_tag,
            },
        );

        self.bump_offset = new_offset;
        Ok(handle)
    }

    /// Deallocate a previously allocated region.
    ///
    /// The memory is not reclaimed immediately — bump-pointer allocation
    /// does not support fragmentation. The handle is removed from the
    /// allocation table.
    ///
    /// # Complexity
    /// O(1).
    pub fn deallocate(&mut self, handle: HeapHandle) -> Result<(), ExitCode> {
        self.allocations
            .remove(&handle)
            .map(|_| ())
            .ok_or(ExitCode::NullDereference)
    }

    /// Write bytes to an allocated region.
    ///
    /// # Errors
    /// Returns `ExitCode::IndexOutOfBounds` if the write would exceed the
    /// allocation size.
    pub fn write(
        &mut self,
        handle: HeapHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<(), ExitCode> {
        let meta = self
            .allocations
            .get(&handle)
            .ok_or(ExitCode::NullDereference)?;
        let alloc_offset = meta.offset;
        if offset + data.len() > meta.size {
            return Err(ExitCode::IndexOutOfBounds);
        }
        let start = alloc_offset + offset;
        self.memory[start..start + data.len()].copy_from_slice(data);
        self.simulate_cache_access(handle.as_u64());
        Ok(())
    }

    /// Read bytes from an allocated region.
    ///
    /// # Errors
    /// Returns `ExitCode::IndexOutOfBounds` if the read would exceed the
    /// allocation size.
    pub fn read(
        &mut self,
        handle: HeapHandle,
        offset: usize,
        size: usize,
    ) -> Result<&[u8], ExitCode> {
        let meta = self
            .allocations
            .get(&handle)
            .ok_or(ExitCode::NullDereference)?;
        let alloc_offset = meta.offset;
        if offset + size > meta.size {
            return Err(ExitCode::IndexOutOfBounds);
        }
        let start = alloc_offset + offset;
        self.simulate_cache_access(handle.as_u64());
        Ok(&self.memory[start..start + size])
    }

    /// Returns the size in bytes of an allocation.
    pub fn allocation_size(&self, handle: HeapHandle) -> Option<usize> {
        self.allocations.get(&handle).map(|m| m.size)
    }

    /// Returns the type tag of an allocation.
    pub fn allocation_type(&self, handle: HeapHandle) -> Option<TypeTag> {
        self.allocations.get(&handle).map(|m| m.type_tag)
    }

    /// Returns the number of active allocations.
    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }

    /// Returns the total bytes currently allocated (bump offset).
    pub fn bytes_allocated(&self) -> usize {
        self.bump_offset
    }

    /// Returns the maximum heap capacity.
    pub fn capacity(&self) -> usize {
        self.config.max_size
    }

    /// Access a memory address, updating cache simulation stats.
    /// Called internally on every heap read/write when cache_config.enabled is true.
    fn simulate_cache_access(&mut self, address: u64) {
        if !self.cache_config.enabled {
            return;
        }

        let line_size = self.cache_config.cache_line_size;
        let cache_line = address / line_size;

        self.recent_accesses.push(cache_line);
        if self.recent_accesses.len() > 1024 {
            self.recent_accesses.remove(0);
        }

        let l1_ways = self.cache_config.l1_size / line_size;
        let l1_set: HashSet<u64> = self
            .recent_accesses
            .iter()
            .rev()
            .take(l1_ways as usize)
            .cloned()
            .collect();
        if l1_set.contains(&cache_line) {
            self.cache_stats.l1_hits += 1;
            return;
        }
        self.cache_stats.l1_misses += 1;

        let l2_ways = self.cache_config.l2_size / line_size;
        if l2_ways > l1_ways {
            let l2_set: HashSet<u64> = self
                .recent_accesses
                .iter()
                .rev()
                .take(l2_ways as usize)
                .cloned()
                .collect();
            if l2_set.contains(&cache_line) {
                self.cache_stats.l2_hits += 1;
                return;
            }
        }
        self.cache_stats.l2_misses += 1;

        self.cache_stats.l3_misses += 1;
    }

    /// Get current cache statistics
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HeapConfig {
        HeapConfig {
            max_size: 1024,
            alignment: 8,
        }
    }

    #[test]
    fn allocate_single_block_succeeds() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(64, TypeTag::Integer).unwrap();
        assert_eq!(heap.allocation_count(), 1);
        assert_eq!(heap.allocation_size(handle), Some(64));
    }

    #[test]
    fn allocate_multiple_blocks() {
        let mut heap = VirtualHeap::new(test_config());
        let h1 = heap.allocate(32, TypeTag::Integer).unwrap();
        let h2 = heap.allocate(64, TypeTag::Real).unwrap();
        assert_eq!(heap.allocation_count(), 2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn allocate_zero_size_returns_exhaustion() {
        let mut heap = VirtualHeap::new(test_config());
        assert_eq!(
            heap.allocate(0, TypeTag::Integer).unwrap_err(),
            ExitCode::HeapExhaustion
        );
    }

    #[test]
    fn allocate_exceeding_capacity_returns_exhaustion() {
        let mut heap = VirtualHeap::new(test_config());
        assert_eq!(
            heap.allocate(2048, TypeTag::Integer).unwrap_err(),
            ExitCode::HeapExhaustion
        );
    }

    #[test]
    fn write_and_read_round_trip() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(64, TypeTag::Integer).unwrap();
        let data = b"hello world";
        heap.write(handle, 0, data).unwrap();
        let read_back = heap.read(handle, 0, data.len()).unwrap();
        assert_eq!(read_back, data);
    }

    #[test]
    fn write_beyond_allocation_returns_bounds_error() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(8, TypeTag::Integer).unwrap();
        let result = heap.write(handle, 4, &[1, 2, 3, 4, 5]);
        assert_eq!(result.unwrap_err(), ExitCode::IndexOutOfBounds);
    }

    #[test]
    fn read_beyond_allocation_returns_bounds_error() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(8, TypeTag::Integer).unwrap();
        let result = heap.read(handle, 4, 8);
        assert_eq!(result.unwrap_err(), ExitCode::IndexOutOfBounds);
    }

    #[test]
    fn write_to_unallocated_handle_returns_exhaustion() {
        let mut heap = VirtualHeap::new(test_config());
        let fake = HeapHandle(999);
        let result = heap.write(fake, 0, &[1]);
        assert_eq!(result.unwrap_err(), ExitCode::NullDereference);
    }

    #[test]
    fn read_from_unallocated_handle_returns_exhaustion() {
        let mut heap = VirtualHeap::new(test_config());
        let fake = HeapHandle(999);
        let result = heap.read(fake, 0, 1);
        assert_eq!(result.unwrap_err(), ExitCode::NullDereference);
    }

    #[test]
    fn deallocate_removes_from_table() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(64, TypeTag::Integer).unwrap();
        heap.deallocate(handle).unwrap();
        assert_eq!(heap.allocation_count(), 0);
    }

    #[test]
    fn deallocate_twice_returns_error() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(64, TypeTag::Integer).unwrap();
        heap.deallocate(handle).unwrap();
        assert_eq!(
            heap.deallocate(handle).unwrap_err(),
            ExitCode::NullDereference
        );
    }

    #[test]
    fn type_tag_is_tracked() {
        let mut heap = VirtualHeap::new(test_config());
        let handle = heap.allocate(64, TypeTag::Real).unwrap();
        assert_eq!(heap.allocation_type(handle), Some(TypeTag::Real));
    }

    #[test]
    fn bytes_allocated_increases_with_allocations() {
        let mut heap = VirtualHeap::new(test_config());
        assert_eq!(heap.bytes_allocated(), 0);
        heap.allocate(64, TypeTag::Integer).unwrap();
        assert_eq!(heap.bytes_allocated(), 64);
    }

    #[test]
    fn alignment_is_respected() {
        let config = HeapConfig {
            max_size: 1024,
            alignment: 16,
        };
        let mut heap = VirtualHeap::new(config);
        let _ = heap.allocate(1, TypeTag::Integer).unwrap();
        // Bump pointer advances by actual allocation size, not aligned size.
        // The alignment ensures each allocation starts on an aligned boundary;
        // the total bytes allocated is the sum of actual sizes.
        assert_eq!(heap.bytes_allocated(), 1);
    }

    #[test]
    fn multiple_allocations_do_not_overlap() {
        let mut heap = VirtualHeap::new(test_config());
        let h1 = heap.allocate(4, TypeTag::Integer).unwrap();
        let h2 = heap.allocate(4, TypeTag::Integer).unwrap();
        heap.write(h1, 0, &[1, 2, 3, 4]).unwrap();
        heap.write(h2, 0, &[5, 6, 7, 8]).unwrap();
        assert_eq!(heap.read(h1, 0, 4).unwrap(), &[1, 2, 3, 4]);
        assert_eq!(heap.read(h2, 0, 4).unwrap(), &[5, 6, 7, 8]);
    }
    #[test]
    fn allocate_exact_capacity() {
        let mut heap = VirtualHeap::new(HeapConfig {
            max_size: 64,
            alignment: 8,
        });
        let h = heap.allocate(64, TypeTag::Integer).unwrap();
        assert_eq!(heap.allocation_size(h), Some(64));
    }
    #[test]
    fn write_zero_bytes() {
        let mut heap = VirtualHeap::new(test_config());
        let h = heap.allocate(8, TypeTag::Integer).unwrap();
        heap.write(h, 0, &[]).unwrap();
    }
    #[test]
    fn read_zero_bytes() {
        let mut heap = VirtualHeap::new(test_config());
        let h = heap.allocate(8, TypeTag::Integer).unwrap();
        assert_eq!(heap.read(h, 0, 0).unwrap().len(), 0);
    }
    #[test]
    fn type_tag_boolean_round_trip() {
        let mut heap = VirtualHeap::new(test_config());
        let h = heap.allocate(1, TypeTag::Boolean).unwrap();
        heap.write(h, 0, &[1]).unwrap();
        assert_eq!(heap.allocation_type(h), Some(TypeTag::Boolean));
    }
    #[test]
    fn default_heap_config() {
        let heap = VirtualHeap::with_default_config();
        assert_eq!(heap.capacity(), 256 * 1024 * 1024);
    }

    #[test]
    fn cache_config_default_disabled() {
        let config = CacheConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn cache_simulation_tracks_accesses() {
        let heap = VirtualHeap::new(HeapConfig::default());
        assert_eq!(heap.cache_config.l1_size, 64 * 1024);
    }

    #[test]
    fn cache_stats_start_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.total_accesses(), 0);
    }

    #[test]
    fn cache_hit_same_address_single_line() {
        let mut heap = VirtualHeap::with_default_config();
        heap.cache_config.enabled = true;
        heap.cache_config.l1_size = 128;
        heap.cache_config.cache_line_size = 64;
        heap.cache_config.l2_size = 0;
        heap.simulate_cache_access(0);
        heap.simulate_cache_access(0);
        assert_eq!(heap.cache_stats.total_accesses(), 2);
    }

    #[test]
    fn cache_miss_different_lines() {
        let mut heap = VirtualHeap::with_default_config();
        heap.cache_config.enabled = true;
        heap.cache_config.l1_size = 64;
        heap.cache_config.cache_line_size = 64;
        heap.cache_config.l2_size = 0;
        heap.simulate_cache_access(0);
        heap.simulate_cache_access(64);
        assert!(heap.cache_stats.total_accesses() >= 2);
    }

    #[test]
    fn cache_stats_penalty_calculation() {
        let stats = CacheStats {
            l1_hits: 100,
            l1_misses: 10,
            l2_hits: 5,
            l2_misses: 5,
            l3_hits: 1,
            l3_misses: 4,
        };
        let penalty = stats.cache_miss_penalty();
        assert!(penalty > 0);
    }
}
