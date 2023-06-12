//! Continuations TODO

use crate::instance::TopOfStackPointer;
use crate::vmcontext::{VMArrayCallFunction, VMFuncRef, VMOpaqueContext, ValRaw};
use crate::{Instance, TrapReason};
use wasmtime_environ::MAXIMUM_CONTINUATION_PAYLOAD_COUNT;
use wasmtime_fibre::{Fiber, FiberStack, Suspend};

/// TODO
#[repr(C)]
pub struct ContinuationObject {
    fiber: *mut Box<Fiber<'static, u128, u128, u128>>,
    args: Vec<u128>,
    payloads: Vec<u128>,
}

/// M:1 Many-to-one mapping. A single ContinuationObject may be
/// referenced by multiple ContinuationReference, though, only one
/// ContinuationReference may hold a non-null reference to the object
/// at a given time.
#[repr(C)]
pub struct ContinuationReference(Option<*mut ContinuationObject>);

/// TODO
#[inline(always)]
pub fn cont_ref_get_obj(contref: *mut u8) -> Result<*mut u8, TrapReason> {
    let contref = contref as *mut ContinuationReference;
    let contopt = unsafe { contref.as_mut().unwrap().0 };
    match contopt {
        None => Err(TrapReason::user_with_backtrace(anyhow::Error::msg(
            "Continuation is already taken",
        ))), // TODO(dhil): presumably we can set things up such that
             // we always read from a non-null reference.
        Some(contptr) => Ok(contptr as *mut u8),
    }
}

/// TODO
#[inline(always)]
pub fn cont_obj_get_args(_instance: &mut Instance, obj: *mut u8) -> *mut u8 {
    let obj = obj as *mut ContinuationObject;
    unsafe { (*obj).args.as_mut_ptr() as *mut u128 as *mut u8 }
}

/// TODO
#[inline(always)]
pub fn cont_obj_get_payloads(_instance: &mut Instance, obj: *mut u8) -> *mut u8 {
    let obj = obj as *mut ContinuationObject;
    unsafe { (*obj).payloads.as_mut_ptr() as *mut u128 as *mut u8 }
}

/// TODO
#[inline(always)]
pub fn cont_new(instance: &mut Instance, func: *mut u8) -> *mut u8 {
    let func = func as *mut VMFuncRef;
    let callee_ctx = unsafe { (*func).vmctx };
    let caller_ctx = VMOpaqueContext::from_vmcontext(instance.vmctx());
    let f = unsafe {
        std::mem::transmute::<
            VMArrayCallFunction,
            unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMOpaqueContext, *mut ValRaw, usize),
        >((*func).array_call)
    };
    let payload_ptr = unsafe { instance.get_typed_continuations_payloads_mut() as *mut ValRaw };
    let fiber = Box::new(
        Fiber::new(
            FiberStack::new(4096).unwrap(),
            move |_first_val: (), _suspend: &Suspend<(), u32, ()>| {
                unsafe {
                    f(
                        callee_ctx,
                        caller_ctx,
                        payload_ptr,
                        MAXIMUM_CONTINUATION_PAYLOAD_COUNT as usize,
                    )
                }
            },
        )
        .unwrap(),
    );
    let ptr: *mut Fiber<'static, (), u32, ()> = Box::into_raw(fiber);
    ptr as *mut u8
}

/// TODO
#[inline(always)]
pub fn resume(instance: &mut Instance, cont: *mut u8) -> Result<u32, TrapReason> {
    let cont = cont as *mut Fiber<'static, (), u32, ()>;
    let cont_stack = unsafe { &cont.as_ref().unwrap().stack() };
    let tsp = TopOfStackPointer::as_raw(instance.tsp());
    unsafe { cont_stack.write_parent(tsp) };
    instance.set_tsp(TopOfStackPointer::from_raw(cont_stack.top().unwrap()));
    unsafe {
        (*(*(*instance.store()).vmruntime_limits())
            .stack_limit
            .get_mut()) = 0
    };
    match unsafe { cont.as_mut().unwrap().resume(()) } {
        Ok(()) => {
            let drop_box: Box<Fiber<_, _, _>> = unsafe { Box::from_raw(cont) };
            drop(drop_box); // I think this would be covered by the close brace below anyway
                            // Store the result.

            // The result of the continuation was written to the first entry of the payload
            // store by virtue of using the array calling trampoline to execute it

            Ok(0) // zero value = return normally.
                  //Ok(9999)
        }
        Err(tag) => {
            // We set the high bit to signal a return via suspend. We
            // encode the tag into the remainder of the integer.
            let signal_mask = 0xf000_0000;
            debug_assert_eq!(tag & signal_mask, 0);
            unsafe {
                let cont_store_ptr = instance.get_typed_continuations_store_mut()
                    as *mut *mut Fiber<'static, (), u32, ()>;
                cont_store_ptr.write(cont)
            };
            Ok(tag | signal_mask)
        } // 0 = suspend //Ok(y),
    }
}

/// TODO
#[inline(always)]
pub fn suspend(instance: &mut Instance, tag_index: u32) {
    let stack_ptr = TopOfStackPointer::as_raw(instance.tsp());
    let parent = unsafe { stack_ptr.cast::<*mut u8>().offset(-2).read() };
    instance.set_tsp(TopOfStackPointer::from_raw(parent));
    let suspend = wasmtime_fibre::unix::Suspend::from_top_ptr(stack_ptr);
    suspend.switch::<(), u32, ()>(wasmtime_fibre::RunResult::Yield(tag_index))
}
