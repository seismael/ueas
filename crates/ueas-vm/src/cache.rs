//! Real K-way set-associative LRU L1/L2/L3 cache simulator over VirtualHeap addresses.

use serde::{Deserialize, Serialize};

/// Configuration for a single cache level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheLevelConfig {
    pub size_bytes: usize,
    pub line_bytes: usize,
    pub associativity: usize,
}

/// Complete L1/L2/L3 cache hierarchy configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheConfig {
    pub l1: CacheLevelConfig,
    pub l2: CacheLevelConfig,
    pub l3: CacheLevelConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1: CacheLevelConfig {
                size_bytes: 32 * 1024,
                line_bytes: 64,
                associativity: 8,
            },
            l2: CacheLevelConfig {
                size_bytes: 256 * 1024,
                line_bytes: 64,
                associativity: 8,
            },
            l3: CacheLevelConfig {
                size_bytes: 2 * 1024 * 1024,
                line_bytes: 64,
                associativity: 16,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheLine {
    tag: u64,
    last_used: u64,
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheLevel {
    line_bytes: usize,
    num_sets: usize,
    associativity: usize,
    lines: Vec<CacheLine>,
    hits: u64,
    misses: u64,
}

impl CacheLevel {
    fn new(cfg: CacheLevelConfig) -> Self {
        let line_bytes = cfg.line_bytes.max(16);
        let associativity = cfg.associativity.max(1);
        let total_lines = (cfg.size_bytes / line_bytes).max(associativity);
        let num_sets = (total_lines / associativity).max(1);
        let lines = vec![
            CacheLine {
                tag: 0,
                last_used: 0,
                valid: false,
            };
            num_sets * associativity
        ];
        Self {
            line_bytes,
            num_sets,
            associativity,
            lines,
            hits: 0,
            misses: 0,
        }
    }

    fn access(&mut self, addr: u64, tick: u64) -> bool {
        let line_addr = addr / (self.line_bytes as u64);
        let set_idx = (line_addr as usize) % self.num_sets;
        let tag = line_addr / (self.num_sets as u64);
        let base = set_idx * self.associativity;
        let slice = &mut self.lines[base..base + self.associativity];

        for line in slice.iter_mut() {
            if line.valid && line.tag == tag {
                line.last_used = tick;
                self.hits += 1;
                return true;
            }
        }

        self.misses += 1;
        let mut victim_idx = 0;
        let mut min_used = u64::MAX;
        for (i, line) in slice.iter().enumerate() {
            if !line.valid {
                victim_idx = i;
                break;
            }
            if line.last_used < min_used {
                min_used = line.last_used;
                victim_idx = i;
            }
        }
        slice[victim_idx] = CacheLine {
            tag,
            last_used: tick,
            valid: true,
        };
        false
    }
}

/// Summary cache performance statistics across L1, L2, and L3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CacheStats {
    pub accesses: u64,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub estimated_memory_cycles: u64,
}

/// Simulated inclusive L1/L2/L3 set-associative cache hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHierarchy {
    l1: CacheLevel,
    l2: CacheLevel,
    l3: CacheLevel,
    tick: u64,
    accesses: u64,
}

impl CacheHierarchy {
    /// Construct a cache hierarchy from `CacheConfig`.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1: CacheLevel::new(config.l1),
            l2: CacheLevel::new(config.l2),
            l3: CacheLevel::new(config.l3),
            tick: 0,
            accesses: 0,
        }
    }

    /// Record a memory access at virtual address `addr` of `size_bytes`, returning the L1 cache line index.
    pub fn access(&mut self, addr: u64, size_bytes: usize) -> u64 {
        let line_size = self.l1.line_bytes as u64;
        let first_line = addr / line_size;
        let last_line = (addr.saturating_add(size_bytes.max(1) as u64 - 1)) / line_size;

        for line in first_line..=last_line {
            let line_addr = line * line_size;
            self.tick += 1;
            self.accesses += 1;
            if !self.l1.access(line_addr, self.tick) && !self.l2.access(line_addr, self.tick) {
                let _ = self.l3.access(line_addr, self.tick);
            }
        }
        first_line
    }

    /// Return aggregated L1/L2/L3 hit/miss statistics.
    pub fn stats(&self) -> CacheStats {
        let cycles =
            self.l1.hits * 4 + self.l2.hits * 12 + self.l3.hits * 40 + self.l3.misses * 200;
        CacheStats {
            accesses: self.accesses,
            l1_hits: self.l1.hits,
            l1_misses: self.l1.misses,
            l2_hits: self.l2.hits,
            l2_misses: self.l2.misses,
            l3_hits: self.l3.hits,
            l3_misses: self.l3.misses,
            estimated_memory_cycles: cycles,
        }
    }
}
