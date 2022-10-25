//! Internal dependency of Wasmtime and Cranelift that defines types for
//! WebAssembly.

pub use wasmparser;

use cranelift_entity::entity_impl;

use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;

mod error;
pub use error::*;

/// WebAssembly value type -- equivalent of `wasmparser`'s Type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmType {
    /// I32 type
    I32,
    /// I64 type
    I64,
    /// F32 type
    F32,
    /// F64 type
    F64,
    /// V128 type
    V128,
    /// Reference type
    Ref(WasmRefType),
}

impl TryFrom<wasmparser::ValType> for WasmType {
    type Error = WasmError;
    fn try_from(ty: wasmparser::ValType) -> Result<Self, Self::Error> {
        use wasmparser::ValType::*;
        match ty {
            I32 => Ok(WasmType::I32),
            I64 => Ok(WasmType::I64),
            F32 => Ok(WasmType::F32),
            F64 => Ok(WasmType::F64),
            V128 => Ok(WasmType::V128),
            Ref(rt) => Ok(WasmType::Ref(WasmRefType::try_from(rt)?)),
        }
    }
}

impl WasmType {
    fn from_val_type(
        ty: wasmparser::ValType,
        types: &[wasmparser::Type],
    ) -> Result<Self, WasmError> {
        use wasmparser::ValType::*;
        match ty {
            I32 => Ok(WasmType::I32),
            I64 => Ok(WasmType::I64),
            F32 => Ok(WasmType::F32),
            F64 => Ok(WasmType::F64),
            V128 => Ok(WasmType::V128),
            Ref(rt) => Ok(WasmType::Ref(WasmRefType::from_ref_type(rt, types)?)),
        }
    }
}

impl From<WasmType> for wasmparser::ValType {
    fn from(ty: WasmType) -> wasmparser::ValType {
        match ty {
            WasmType::I32 => wasmparser::ValType::I32,
            WasmType::I64 => wasmparser::ValType::I64,
            WasmType::F32 => wasmparser::ValType::F32,
            WasmType::F64 => wasmparser::ValType::F64,
            WasmType::V128 => wasmparser::ValType::V128,
            WasmType::Ref(rt) => wasmparser::ValType::Ref(wasmparser::RefType::from(rt)),
        }
    }
}

impl fmt::Display for WasmType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmType::I32 => write!(f, "i32"),
            WasmType::I64 => write!(f, "i64"),
            WasmType::F32 => write!(f, "f32"),
            WasmType::F64 => write!(f, "f64"),
            WasmType::V128 => write!(f, "v128"),
            WasmType::Ref(rt) => write!(f, "{}", rt),
        }
    }
}

/// WebAssembly reference type -- equivalent of `wasmparser`'s RefType
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WasmRefType {
    pub nullable: bool,
    pub heap_type: WasmHeapType,
}

pub const WASM_EXTERN_REF: WasmRefType = WasmRefType {
    nullable: true,
    heap_type: WasmHeapType::Extern,
};

pub const WASM_FUNC_REF: WasmRefType = WasmRefType {
    nullable: true,
    heap_type: WasmHeapType::Func,
};

impl TryFrom<wasmparser::RefType> for WasmRefType {
    type Error = WasmError;
    fn try_from(
        wasmparser::RefType {
            nullable,
            heap_type,
        }: wasmparser::RefType,
    ) -> Result<Self, Self::Error> {
        Ok(WasmRefType {
            nullable,
            heap_type: WasmHeapType::try_from(heap_type)?,
        })
    }
}

impl WasmRefType {
    fn from_ref_type(
        wasmparser::RefType {
            nullable,
            heap_type,
        }: wasmparser::RefType,
        types: &[wasmparser::Type],
    ) -> Result<Self, WasmError> {
        Ok(WasmRefType {
            nullable,
            heap_type: WasmHeapType::from_heap_type(heap_type, types)?,
        })
    }
}

impl From<WasmRefType> for wasmparser::RefType {
    fn from(
        WasmRefType {
            nullable,
            heap_type,
        }: WasmRefType,
    ) -> wasmparser::RefType {
        wasmparser::RefType {
            nullable,
            heap_type: wasmparser::HeapType::from(heap_type),
        }
    }
}

impl fmt::Display for WasmRefType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (*self).heap_type {
            WasmHeapType::Extern => write!(f, "externref"),
            WasmHeapType::Func => write!(f, "funcref"),
            ht => {
                if (*self).nullable {
                    write!(f, "(ref null {})", ht)
                } else {
                    write!(f, "(ref {})", ht)
                }
            }
        }
    }
}

/// WebAssembly heap type -- equivalent of `wasmparser`'s HeapType
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmHeapType {
    Bot,
    Func,
    Extern,
    FuncIndex(u32),
    ContIndex(u32),
}

impl TryFrom<wasmparser::HeapType> for WasmHeapType {
    type Error = WasmError;
    fn try_from(ht: wasmparser::HeapType) -> Result<Self, Self::Error> {
        use wasmparser::HeapType::*;
        match ht {
            Bot => Ok(WasmHeapType::Bot),
            Func => Ok(WasmHeapType::Func),
            Extern => Ok(WasmHeapType::Extern),
            TypedFunc(_) => todo!(),
        }
    }
}

impl WasmHeapType {
    fn from_heap_type(
        ht: wasmparser::HeapType,
        types: &[wasmparser::Type],
    ) -> Result<Self, WasmError> {
        use wasmparser::HeapType::*;
        match ht {
            Bot => Ok(WasmHeapType::Bot),
            Func => Ok(WasmHeapType::Func),
            Extern => Ok(WasmHeapType::Extern),
            TypedFunc(i) => match types[u32::from(i) as usize] {
                wasmparser::Type::Func(_) => Ok(WasmHeapType::FuncIndex(u32::from(i))),
                wasmparser::Type::Cont(_) => Ok(WasmHeapType::ContIndex(u32::from(i))),
            },
        }
    }
}

impl From<WasmHeapType> for wasmparser::HeapType {
    fn from(ht: WasmHeapType) -> wasmparser::HeapType {
        match ht {
            WasmHeapType::Bot => wasmparser::HeapType::Bot,
            WasmHeapType::Func => wasmparser::HeapType::Func,
            WasmHeapType::Extern => wasmparser::HeapType::Extern,
            WasmHeapType::FuncIndex(i) => i.try_into().unwrap(),
            WasmHeapType::ContIndex(i) => i.try_into().unwrap(),
        }
    }
}

impl fmt::Display for WasmHeapType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmHeapType::Bot => write!(f, "bot"),
            WasmHeapType::Func => write!(f, "func"),
            WasmHeapType::Extern => write!(f, "extern"),
            WasmHeapType::FuncIndex(i) | WasmHeapType::ContIndex(i) => write!(f, "{}", i),
        }
    }
}

/// WebAssembly function type -- equivalent of `wasmparser`'s FuncType.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WasmFuncType {
    params: Box<[WasmType]>,
    externref_params_count: usize,
    returns: Box<[WasmType]>,
    externref_returns_count: usize,
}

impl WasmFuncType {
    #[inline]
    pub fn new(params: Box<[WasmType]>, returns: Box<[WasmType]>) -> Self {
        let externref_params_count = params
            .iter()
            .filter(|p| match **p {
                WasmType::Ref(rt) => rt.heap_type == WasmHeapType::Extern,
                _ => false,
            })
            .count();
        let externref_returns_count = returns
            .iter()
            .filter(|r| match **r {
                WasmType::Ref(rt) => rt.heap_type == WasmHeapType::Extern,
                _ => false,
            })
            .count();
        WasmFuncType {
            params,
            externref_params_count,
            returns,
            externref_returns_count,
        }
    }

    /// Function params types.
    #[inline]
    pub fn params(&self) -> &[WasmType] {
        &self.params
    }

    /// How many `externref`s are in this function's params?
    #[inline]
    pub fn externref_params_count(&self) -> usize {
        self.externref_params_count
    }

    /// Returns params types.
    #[inline]
    pub fn returns(&self) -> &[WasmType] {
        &self.returns
    }

    /// How many `externref`s are in this function's returns?
    #[inline]
    pub fn externref_returns_count(&self) -> usize {
        self.externref_returns_count
    }

    /// Resolve indices and convert to WasmType
    pub fn from_func_type(
        ty: wasmparser::FuncType,
        types: &[wasmparser::Type],
    ) -> Result<Self, WasmError> {
        let params = ty
            .params()
            .iter()
            .copied()
            .map(|t| WasmType::from_val_type(t, types))
            .collect::<Result<_, WasmError>>()?;
        let returns = ty
            .results()
            .iter()
            .copied()
            .map(|t| WasmType::from_val_type(t, types))
            .collect::<Result<_, WasmError>>()?;
        Ok(Self::new(params, returns))
    }
}

impl TryFrom<wasmparser::FuncType> for WasmFuncType {
    type Error = WasmError;
    fn try_from(ty: wasmparser::FuncType) -> Result<Self, Self::Error> {
        let params = ty
            .params()
            .iter()
            .copied()
            .map(WasmType::try_from)
            .collect::<Result<_, Self::Error>>()?;
        let returns = ty
            .results()
            .iter()
            .copied()
            .map(WasmType::try_from)
            .collect::<Result<_, Self::Error>>()?;
        Ok(Self::new(params, returns))
    }
}

/// Index type of a function (imported or defined) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct FuncIndex(u32);
entity_impl!(FuncIndex);

/// Index type of a defined function inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DefinedFuncIndex(u32);
entity_impl!(DefinedFuncIndex);

/// Index type of a defined table inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DefinedTableIndex(u32);
entity_impl!(DefinedTableIndex);

/// Index type of a defined memory inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DefinedMemoryIndex(u32);
entity_impl!(DefinedMemoryIndex);

/// Index type of a defined memory inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct OwnedMemoryIndex(u32);
entity_impl!(OwnedMemoryIndex);

/// Index type of a defined global inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DefinedGlobalIndex(u32);
entity_impl!(DefinedGlobalIndex);

/// Index type of a table (imported or defined) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TableIndex(u32);
entity_impl!(TableIndex);

/// Index type of a global variable (imported or defined) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct GlobalIndex(u32);
entity_impl!(GlobalIndex);

/// Index type of a linear memory (imported or defined) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct MemoryIndex(u32);
entity_impl!(MemoryIndex);

/// Index type of a signature (imported or defined) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct SignatureIndex(u32);
entity_impl!(SignatureIndex);

/// Index type of a passive data segment inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DataIndex(u32);
entity_impl!(DataIndex);

/// Index type of a passive element segment inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct ElemIndex(u32);
entity_impl!(ElemIndex);

/// Index type of a type inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TypeIndex(u32);
entity_impl!(TypeIndex);

/// Index type of an event inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TagIndex(u32);
entity_impl!(TagIndex);

/// An index of an entity.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum EntityIndex {
    /// Function index.
    Function(FuncIndex),
    /// Table index.
    Table(TableIndex),
    /// Memory index.
    Memory(MemoryIndex),
    /// Global index.
    Global(GlobalIndex),
}

impl From<FuncIndex> for EntityIndex {
    fn from(idx: FuncIndex) -> EntityIndex {
        EntityIndex::Function(idx)
    }
}

impl From<TableIndex> for EntityIndex {
    fn from(idx: TableIndex) -> EntityIndex {
        EntityIndex::Table(idx)
    }
}

impl From<MemoryIndex> for EntityIndex {
    fn from(idx: MemoryIndex) -> EntityIndex {
        EntityIndex::Memory(idx)
    }
}

impl From<GlobalIndex> for EntityIndex {
    fn from(idx: GlobalIndex) -> EntityIndex {
        EntityIndex::Global(idx)
    }
}

/// A type of an item in a wasm module where an item is typically something that
/// can be exported.
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityType {
    /// A global variable with the specified content type
    Global(Global),
    /// A linear memory with the specified limits
    Memory(Memory),
    /// An event definition.
    Tag(Tag),
    /// A table with the specified element type and limits
    Table(Table),
    /// A function type where the index points to the type section and records a
    /// function signature.
    Function(SignatureIndex),
}

/// A WebAssembly global.
///
/// Note that we record both the original Wasm type and the Cranelift IR type
/// used to represent it. This is because multiple different kinds of Wasm types
/// might be represented with the same Cranelift IR type. For example, both a
/// Wasm `i64` and a `funcref` might be represented with a Cranelift `i64` on
/// 64-bit architectures, and when GC is not required for func refs.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Global {
    /// The Wasm type of the value stored in the global.
    pub wasm_ty: crate::WasmType,
    /// A flag indicating whether the value may change at runtime.
    pub mutability: bool,
    /// The source of the initial value.
    pub initializer: GlobalInit,
}

/// Globals are initialized via the `const` operators or by referring to another import.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum GlobalInit {
    /// An `i32.const`.
    I32Const(i32),
    /// An `i64.const`.
    I64Const(i64),
    /// An `f32.const`.
    F32Const(u32),
    /// An `f64.const`.
    F64Const(u64),
    /// A `vconst`.
    V128Const(u128),
    /// A `global.get` of another global.
    GetGlobal(GlobalIndex),
    /// A `ref.null`.
    RefNullConst,
    /// A `ref.func <index>`.
    RefFunc(FuncIndex),
    ///< The global is imported from, and thus initialized by, a different module.
    Import,
}

impl Global {
    /// Creates a new `Global` type from wasmparser's representation.
    pub fn new(ty: wasmparser::GlobalType, initializer: GlobalInit) -> WasmResult<Global> {
        Ok(Global {
            wasm_ty: ty.content_type.try_into()?,
            mutability: ty.mutable,
            initializer,
        })
    }
}

/// WebAssembly table.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// The table elements' Wasm type.
    pub wasm_ty: WasmRefType,
    /// The minimum number of elements in the table.
    pub minimum: u32,
    /// The maximum number of elements in the table.
    pub maximum: Option<u32>,
}

impl TryFrom<wasmparser::TableType> for Table {
    type Error = WasmError;

    fn try_from(ty: wasmparser::TableType) -> WasmResult<Table> {
        Ok(Table {
            wasm_ty: ty.element_type.try_into()?,
            minimum: ty.initial,
            maximum: ty.maximum,
        })
    }
}

/// WebAssembly linear memory.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// The minimum number of pages in the memory.
    pub minimum: u64,
    /// The maximum number of pages in the memory.
    pub maximum: Option<u64>,
    /// Whether the memory may be shared between multiple threads.
    pub shared: bool,
    /// Whether or not this is a 64-bit memory
    pub memory64: bool,
}

impl From<wasmparser::MemoryType> for Memory {
    fn from(ty: wasmparser::MemoryType) -> Memory {
        Memory {
            minimum: ty.initial,
            maximum: ty.maximum,
            shared: ty.shared,
            memory64: ty.memory64,
        }
    }
}

/// WebAssembly event.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// The event signature type.
    pub ty: TypeIndex,
}

impl From<wasmparser::TagType> for Tag {
    fn from(ty: wasmparser::TagType) -> Tag {
        match ty.kind {
            wasmparser::TagKind::Exception => Tag {
                ty: TypeIndex::from_u32(ty.func_type_idx),
            },
        }
    }
}
