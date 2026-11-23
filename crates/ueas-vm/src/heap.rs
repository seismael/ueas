//! Generational VirtualHeap with real byte accounting and eager scope drop.

use crate::trap::{TrapCode, TrapError};
use crate::value::{HeapHandle, RuntimeKey, RuntimeValue};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// Real collection/record objects stored in `VirtualHeap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeapObject {
    List(Vec<RuntimeValue>),
    Set(IndexSet<RuntimeKey>),
    Map(IndexMap<RuntimeKey, RuntimeValue>),
    Graph {
        directed: bool,
        adj: IndexMap<RuntimeKey, Vec<(RuntimeKey, RuntimeValue)>>,
    },
    Matrix {
        rows: usize,
        cols: usize,
        data: Vec<RuntimeValue>,
    },
    Record {
        type_name: String,
        fields: IndexMap<String, RuntimeValue>,
    },
}

impl HeapObject {
    /// Compute the deterministic byte footprint of this heap object.
    pub fn byte_size(&self) -> usize {
        match self {
            Self::List(items) => 32 + items.iter().map(RuntimeValue::shallow_bytes).sum::<usize>(),
            Self::Set(items) => 48 + items.len() * 24,
            Self::Map(map) => 48 + map.values().map(|v| 24 + v.shallow_bytes()).sum::<usize>(),
            Self::Graph { adj, .. } => {
                64 + adj
                    .values()
                    .map(|edges| 24 + edges.len() * 32)
                    .sum::<usize>()
            }
            Self::Matrix { data, .. } => {
                40 + data.iter().map(RuntimeValue::shallow_bytes).sum::<usize>()
            }
            Self::Record { type_name, fields } => {
                32 + type_name.len()
                    + fields
                        .iter()
                        .map(|(k, v)| k.len() + v.shallow_bytes())
                        .sum::<usize>()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Slot {
    generation: u32,
    refcount: u32,
    obj: Option<HeapObject>,
    byte_size: usize,
    base_vaddr: u64,
}

/// Summary statistics for `VirtualHeap`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeapStats {
    pub live_bytes: usize,
    pub peak_live_bytes: usize,
    pub total_allocations: u64,
    pub total_frees: u64,
}

/// Generational heap isolated from host memory, enforcing `heap_budget_bytes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualHeap {
    slots: Vec<Slot>,
    free_list: Vec<u32>,
    next_vaddr: u64,
    live_bytes: usize,
    peak_live_bytes: usize,
    total_allocations: u64,
    total_frees: u64,
    budget_bytes: usize,
}

impl VirtualHeap {
    /// Create a fresh `VirtualHeap` with the given byte budget.
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            next_vaddr: 0x1_0000,
            live_bytes: 0,
            peak_live_bytes: 0,
            total_allocations: 0,
            total_frees: 0,
            budget_bytes,
        }
    }

    /// Allocate a `HeapObject` and return its generational `HeapHandle`.
    pub fn alloc(&mut self, obj: HeapObject) -> Result<HeapHandle, TrapError> {
        let bytes = obj.byte_size();
        if self.live_bytes.saturating_add(bytes) > self.budget_bytes {
            return Err(TrapError::new(
                TrapCode::HeapBudgetExceeded,
                format!(
                    "heap budget exceeded: live {} + alloc {} > budget {}",
                    self.live_bytes, bytes, self.budget_bytes
                ),
                0,
                0,
                None,
            ));
        }

        self.live_bytes += bytes;
        if self.live_bytes > self.peak_live_bytes {
            self.peak_live_bytes = self.live_bytes;
        }
        self.total_allocations += 1;

        let base_vaddr = self.next_vaddr;
        let stride = ((bytes as u64).max(64) + 63) & !63;
        self.next_vaddr = self.next_vaddr.saturating_add(stride);

        if let Some(idx) = self.free_list.pop() {
            let slot = &mut self.slots[idx as usize];
            slot.obj = Some(obj);
            slot.refcount = 1;
            slot.byte_size = bytes;
            slot.base_vaddr = base_vaddr;
            Ok(HeapHandle {
                index: idx,
                generation: slot.generation,
            })
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 1,
                refcount: 1,
                obj: Some(obj),
                byte_size: bytes,
                base_vaddr,
            });
            Ok(HeapHandle {
                index: idx,
                generation: 1,
            })
        }
    }

    /// Eagerly drop a heap allocation, reclaiming bytes and bumping generation.
    pub fn free(&mut self, handle: HeapHandle) -> Result<(), TrapError> {
        let slot = self.slots.get_mut(handle.index as usize).ok_or_else(|| {
            TrapError::new(TrapCode::UseAfterFree, "invalid heap handle", 0, 0, None)
        })?;
        if slot.generation != handle.generation || slot.obj.is_none() {
            return Err(TrapError::new(
                TrapCode::UseAfterFree,
                "double-free or stale generational handle",
                0,
                0,
                None,
            ));
        }
        let freed_obj = slot.obj.take();
        slot.refcount = 0;
        slot.generation = slot.generation.wrapping_add(1);
        self.live_bytes = self.live_bytes.saturating_sub(slot.byte_size);
        slot.byte_size = 0;
        self.total_frees += 1;
        self.free_list.push(handle.index);
        if let Some(obj) = freed_obj {
            match obj {
                HeapObject::List(items) | HeapObject::Matrix { data: items, .. } => {
                    for it in &items {
                        self.release_value(it);
                    }
                }
                HeapObject::Map(map) => {
                    for v in map.values() {
                        self.release_value(v);
                    }
                }
                HeapObject::Record { fields, .. } => {
                    for v in fields.values() {
                        self.release_value(v);
                    }
                }
                HeapObject::Set(_) | HeapObject::Graph { .. } => {}
            }
        }
        Ok(())
    }

    /// Increment reference count for CoW sharing in O(1) time and 0 extra bytes.
    pub fn retain(&mut self, handle: HeapHandle) {
        if let Some(slot) = self.slots.get_mut(handle.index as usize) {
            if slot.generation == handle.generation && slot.obj.is_some() {
                slot.refcount = slot.refcount.saturating_add(1);
            }
        }
    }

    /// Retain any heap handles inside a `RuntimeValue` in O(1) time.
    pub fn retain_value(&mut self, val: &RuntimeValue) {
        match val {
            RuntimeValue::Handle(h) => self.retain(*h),
            RuntimeValue::OptionVal(Some(inner))
            | RuntimeValue::ResultVal(Ok(inner))
            | RuntimeValue::ResultVal(Err(inner)) => self.retain_value(inner),
            RuntimeValue::Tuple(items) => {
                for it in items {
                    self.retain_value(it);
                }
            }
            _ => {}
        }
    }

    /// Decrement reference count and free if it reaches zero.
    pub fn release(&mut self, handle: HeapHandle) {
        let should_free = if let Some(slot) = self.slots.get_mut(handle.index as usize) {
            if slot.generation == handle.generation && slot.obj.is_some() {
                slot.refcount = slot.refcount.saturating_sub(1);
                slot.refcount == 0
            } else {
                false
            }
        } else {
            false
        };
        if should_free {
            let _ = self.free(handle);
        }
    }

    /// Release any heap handles inside a `RuntimeValue`.
    pub fn release_value(&mut self, val: &RuntimeValue) {
        match val {
            RuntimeValue::Handle(h) => self.release(*h),
            RuntimeValue::OptionVal(Some(inner))
            | RuntimeValue::ResultVal(Ok(inner))
            | RuntimeValue::ResultVal(Err(inner)) => self.release_value(inner),
            RuntimeValue::Tuple(items) => {
                for it in items {
                    self.release_value(it);
                }
            }
            _ => {}
        }
    }

    /// Copy-on-Write: if `refcount > 1`, clone the `HeapObject` into a fresh exclusive slot.
    pub fn ensure_unique(&mut self, handle: HeapHandle) -> Result<HeapHandle, TrapError> {
        let (needs_clone, obj_clone) = {
            let slot = self.slots.get(handle.index as usize).ok_or_else(|| {
                TrapError::new(TrapCode::UseAfterFree, "invalid heap handle", 0, 0, None)
            })?;
            if slot.generation != handle.generation || slot.obj.is_none() {
                return Err(TrapError::new(
                    TrapCode::UseAfterFree,
                    "use-after-free: stale generational handle",
                    0,
                    0,
                    None,
                ));
            }
            if slot.refcount <= 1 {
                return Ok(handle);
            }
            (true, slot.obj.clone())
        };
        if needs_clone {
            if let Some(obj) = obj_clone {
                self.release(handle);
                return self.alloc(obj);
            }
        }
        Ok(handle)
    }

    /// Borrow a `HeapObject` immutably and return its base virtual address.
    pub fn get(&self, handle: HeapHandle) -> Result<(&HeapObject, u64), TrapError> {
        let slot = self.slots.get(handle.index as usize).ok_or_else(|| {
            TrapError::new(TrapCode::UseAfterFree, "invalid heap handle", 0, 0, None)
        })?;
        if slot.generation != handle.generation {
            return Err(TrapError::new(
                TrapCode::UseAfterFree,
                "use-after-free: stale generational handle",
                0,
                0,
                None,
            ));
        }
        let obj = slot.obj.as_ref().ok_or_else(|| {
            TrapError::new(
                TrapCode::UseAfterFree,
                "use-after-free: freed object",
                0,
                0,
                None,
            )
        })?;
        Ok((obj, slot.base_vaddr))
    }

    /// Borrow a `HeapObject` mutably and recompute live bytes after mutation.
    pub fn with_mut<R>(
        &mut self,
        handle: HeapHandle,
        f: impl FnOnce(&mut HeapObject, u64) -> Result<R, TrapError>,
    ) -> Result<R, TrapError> {
        let (old_size, base_vaddr) = {
            let slot = self.slots.get(handle.index as usize).ok_or_else(|| {
                TrapError::new(TrapCode::UseAfterFree, "invalid heap handle", 0, 0, None)
            })?;
            if slot.generation != handle.generation || slot.obj.is_none() {
                return Err(TrapError::new(
                    TrapCode::UseAfterFree,
                    "use-after-free: stale generational handle",
                    0,
                    0,
                    None,
                ));
            }
            (slot.byte_size, slot.base_vaddr)
        };

        let slot = &mut self.slots[handle.index as usize];
        let obj = slot
            .obj
            .as_mut()
            .ok_or_else(|| TrapError::new(TrapCode::UseAfterFree, "use-after-free", 0, 0, None))?;
        let res = f(obj, base_vaddr)?;
        let new_size = obj.byte_size();
        let next_live = self
            .live_bytes
            .saturating_sub(old_size)
            .saturating_add(new_size);
        if next_live > self.budget_bytes {
            return Err(TrapError::new(
                TrapCode::HeapBudgetExceeded,
                format!(
                    "heap budget exceeded during mutation: {next_live} > {}",
                    self.budget_bytes
                ),
                0,
                0,
                None,
            ));
        }
        slot.byte_size = new_size;
        self.live_bytes = next_live;
        if self.live_bytes > self.peak_live_bytes {
            self.peak_live_bytes = self.live_bytes;
        }
        Ok(res)
    }

    /// Deep-clone a `RuntimeValue` (allocating fresh heap objects for MVS copy-on-pass/assign).
    pub fn deep_clone_value(&mut self, val: &RuntimeValue) -> Result<RuntimeValue, TrapError> {
        match val {
            RuntimeValue::OptionVal(Some(inner)) => Ok(RuntimeValue::OptionVal(Some(Box::new(
                self.deep_clone_value(inner)?,
            )))),
            RuntimeValue::ResultVal(Ok(inner)) => Ok(RuntimeValue::ResultVal(Ok(Box::new(
                self.deep_clone_value(inner)?,
            )))),
            RuntimeValue::ResultVal(Err(inner)) => Ok(RuntimeValue::ResultVal(Err(Box::new(
                self.deep_clone_value(inner)?,
            )))),
            RuntimeValue::Tuple(items) => {
                let mut cloned = Vec::with_capacity(items.len());
                for item in items {
                    cloned.push(self.deep_clone_value(item)?);
                }
                Ok(RuntimeValue::Tuple(cloned))
            }
            RuntimeValue::Handle(h) => {
                let obj = self.get(*h)?.0.clone();
                let cloned_obj = match obj {
                    HeapObject::List(items) => {
                        let mut out = Vec::with_capacity(items.len());
                        for item in items {
                            out.push(self.deep_clone_value(&item)?);
                        }
                        HeapObject::List(out)
                    }
                    HeapObject::Set(s) => HeapObject::Set(s),
                    HeapObject::Map(m) => {
                        let mut out = IndexMap::with_capacity(m.len());
                        for (k, v) in m {
                            out.insert(k, self.deep_clone_value(&v)?);
                        }
                        HeapObject::Map(out)
                    }
                    HeapObject::Graph { directed, adj } => HeapObject::Graph { directed, adj },
                    HeapObject::Matrix { rows, cols, data } => {
                        let mut out = Vec::with_capacity(data.len());
                        for item in data {
                            out.push(self.deep_clone_value(&item)?);
                        }
                        HeapObject::Matrix {
                            rows,
                            cols,
                            data: out,
                        }
                    }
                    HeapObject::Record { type_name, fields } => {
                        let mut out = IndexMap::with_capacity(fields.len());
                        for (k, v) in fields {
                            out.insert(k, self.deep_clone_value(&v)?);
                        }
                        HeapObject::Record {
                            type_name,
                            fields: out,
                        }
                    }
                };
                Ok(RuntimeValue::Handle(self.alloc(cloned_obj)?))
            }
            other => Ok(other.clone()),
        }
    }

    /// Return current heap accounting statistics.
    pub fn stats(&self) -> HeapStats {
        HeapStats {
            live_bytes: self.live_bytes,
            peak_live_bytes: self.peak_live_bytes,
            total_allocations: self.total_allocations,
            total_frees: self.total_frees,
        }
    }
}
