//! Helper functions and structures for the translation.
use crate::environ::TargetEnvironment;
use crate::WasmResult;
use core::convert::TryInto;
use core::u32;
use cranelift_codegen::ir;
use cranelift_frontend::FunctionBuilder;
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use wasmparser::{FuncValidator, WasmFuncType, WasmModuleResources};

/// WebAssembly table element. Can be a function or a scalar type.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum TableElementType {
    /// A scalar type.
    Val(ir::Type),
    /// A function.
    Func,
}

/// Helper function translating wasmparser types to Cranelift types when possible.
pub fn type_to_type<PE: TargetEnvironment + ?Sized>(
    ty: wasmparser::ValType,
    environ: &PE,
) -> WasmResult<ir::Type> {
    match ty {
        wasmparser::ValType::I32 => Ok(ir::types::I32),
        wasmparser::ValType::I64 => Ok(ir::types::I64),
        wasmparser::ValType::F32 => Ok(ir::types::F32),
        wasmparser::ValType::F64 => Ok(ir::types::F64),
        wasmparser::ValType::V128 => Ok(ir::types::I8X16),
        wasmparser::ValType::Ref(rt) => Ok(environ.reference_type(rt.heap_type.try_into()?)), // TODO(dhil) fixme: verify this is indeed the right thing to do.
    }
}

/// Helper function translating wasmparser possible table types to Cranelift types when possible,
/// or None for Func tables.
pub fn tabletype_to_type<PE: TargetEnvironment + ?Sized>(
    ty: wasmparser::ValType,
    environ: &PE,
) -> WasmResult<Option<ir::Type>> {
    match ty {
        wasmparser::ValType::I32 => Ok(Some(ir::types::I32)),
        wasmparser::ValType::I64 => Ok(Some(ir::types::I64)),
        wasmparser::ValType::F32 => Ok(Some(ir::types::F32)),
        wasmparser::ValType::F64 => Ok(Some(ir::types::F64)),
        wasmparser::ValType::V128 => Ok(Some(ir::types::I8X16)),
        wasmparser::ValType::Ref(rt) => {
            match rt.heap_type {
                wasmparser::HeapType::Extern => {
                    Ok(Some(environ.reference_type(rt.heap_type.try_into()?)))
                }
                _ => Ok(None), // TODO(dhil) fixme: verify this is indeed the right thing to do.
            }
        }
    }
}

/// Get the parameter and result types for the given Wasm blocktype.
pub fn blocktype_params_results<'a, T>(
    validator: &'a FuncValidator<T>,
    ty: wasmparser::BlockType,
) -> WasmResult<(
    impl ExactSizeIterator<Item = wasmparser::ValType> + Clone + 'a,
    impl ExactSizeIterator<Item = wasmparser::ValType> + Clone + 'a,
)>
where
    T: WasmModuleResources,
{
    return Ok(match ty {
        wasmparser::BlockType::Empty => {
            let params: &'static [wasmparser::ValType] = &[];
            // If we care about not allocating, surely we can type munge more.
            // But, it is midnight
            let results: std::vec::Vec<wasmparser::ValType> = vec![];
            (
                itertools::Either::Left(params.iter().copied()),
                itertools::Either::Left(results.into_iter()),
            )
        }
        wasmparser::BlockType::Type(ty) => {
            let params: &'static [wasmparser::ValType] = &[];
            let results: std::vec::Vec<wasmparser::ValType> = vec![ty.clone()];
            (
                itertools::Either::Left(params.iter().copied()),
                itertools::Either::Left(results.into_iter()),
            )
        }
        wasmparser::BlockType::FuncType(ty_index) => {
            let ty = validator
                .resources()
                .func_type_at(ty_index)
                .expect("should be valid");
            (
                itertools::Either::Right(ty.inputs()),
                itertools::Either::Right(ty.outputs()),
            )
        }
    });
}

/// Create a `Block` with the given Wasm parameters.
pub fn block_with_params<PE: TargetEnvironment + ?Sized>(
    builder: &mut FunctionBuilder,
    params: impl IntoIterator<Item = wasmparser::ValType>,
    environ: &PE,
) -> WasmResult<ir::Block> {
    let block = builder.create_block();
    for ty in params {
        match ty {
            wasmparser::ValType::I32 => {
                builder.append_block_param(block, ir::types::I32);
            }
            wasmparser::ValType::I64 => {
                builder.append_block_param(block, ir::types::I64);
            }
            wasmparser::ValType::F32 => {
                builder.append_block_param(block, ir::types::F32);
            }
            wasmparser::ValType::F64 => {
                builder.append_block_param(block, ir::types::F64);
            }
            wasmparser::ValType::Ref(rt) => {
                // TODO: huge warning!  this is bypassing the fact that
                // WasmHeapType needs additional information to determine the
                // true type of a wasmparser index.  We sidestep it by saying
                // that **ON x86-64 LINUX**, we assume that *both typed functions
                // and continuations have the same reference type as untyped
                // functions*.  On other platforms (or even this platform in
                // the future), *this may be incorrect*
                let ht = match rt.heap_type {
                    wasmparser::HeapType::TypedFunc(_) => wasmparser::HeapType::Func,
                    x => x,
                };
                builder.append_block_param(block, environ.reference_type(ht.try_into()?));
            }
            wasmparser::ValType::V128 => {
                builder.append_block_param(block, ir::types::I8X16);
            }
        }
    }
    Ok(block)
}

/// Turns a `wasmparser` `f32` into a `Cranelift` one.
pub fn f32_translation(x: wasmparser::Ieee32) -> ir::immediates::Ieee32 {
    ir::immediates::Ieee32::with_bits(x.bits())
}

/// Turns a `wasmparser` `f64` into a `Cranelift` one.
pub fn f64_translation(x: wasmparser::Ieee64) -> ir::immediates::Ieee64 {
    ir::immediates::Ieee64::with_bits(x.bits())
}

/// Special VMContext value label. It is tracked as 0xffff_fffe label.
pub fn get_vmctx_value_label() -> ir::ValueLabel {
    const VMCTX_LABEL: u32 = 0xffff_fffe;
    ir::ValueLabel::from_u32(VMCTX_LABEL)
}
