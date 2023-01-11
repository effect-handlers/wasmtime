use crate::instance::Instance;
use crate::vmcontext::{VMCallerCheckedAnyfunc, VMContext};
use crate::{prepare_host_to_wasm_trampoline, TrapReason, VMFunctionBody, VMOpaqueContext};
use std::mem;
use wasmtime_fiber::{Fiber, FiberStack, Suspend};

type Resume = ();
type ContFiber = Fiber<'static, (), (), ()>;
#[repr(transparent)]
pub struct Continuation(*mut ContFiber);
impl Continuation {
    pub fn new(vmctx: *mut VMContext, func: *mut VMCallerCheckedAnyfunc) -> Self {
        let fiber = Box::new(
            Fiber::new(FiberStack::new(4096).unwrap(), unsafe {
                move |first_resumption: (), suspend: &Suspend<_, (), _>| {
                    let trampoline = mem::transmute::<
                        *const VMFunctionBody,
                        unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMContext),
                    >((*func).func_ptr.as_ptr());
                    let trampoline = prepare_host_to_wasm_trampoline(vmctx, trampoline);
                    trampoline((*func).vmctx, vmctx);
                }
            })
            .unwrap(),
        );
        let ptr: *mut ContFiber = Box::into_raw(fiber);
        Self(ptr)
    }
    pub fn resume(&self, vmctx: *mut VMContext) -> Result<Resume, ()> {
        unsafe {
            let inst = vmctx.as_mut().unwrap().instance_mut();
            let cont_stack = &self.0.as_ref().unwrap().stack;
            cont_stack.write_parent(inst.tsp());
            inst.set_tsp(cont_stack.top().unwrap());
            (*(*(*(*vmctx).instance().store()).vmruntime_limits())
                .stack_limit
                .get_mut()) = 0;
            self.0.as_mut().unwrap().resume(())
        }
    }
    pub unsafe fn from_erased_ptr(ptr: *mut u8) -> Self {
        Self(ptr as _)
    }
    pub fn as_erased_ptr(self) -> *mut u8 {
        self.0 as _
    }
}
