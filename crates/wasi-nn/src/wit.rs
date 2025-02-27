//! Implements the `wasi-nn` API for the WIT ("preview2") ABI.
//!
//! Note that `wasi-nn` is not yet included in an official "preview2" world
//! (though it could be) so by "preview2" here we mean that this can be called
//! with the component model's canonical ABI.
//!
//! This module exports its [`types`] for use throughout the crate and the
//! [`ML`] object, which exposes [`ML::add_to_linker`]. To implement all of
//! this, this module proceeds in steps:
//! 1. generate all of the WIT glue code into a `gen::*` namespace
//! 2. wire up the `gen::*` glue to the context state, delegating actual
//!    computation to a [`Backend`]
//! 3. convert some types
//!
//! [`Backend`]: crate::backend::Backend
//! [`types`]: crate::wit::types

use crate::{backend::BackendKind, ctx::UsageError, WasiNnCtx};

/// Generate the traits and types from the `wasi-nn` WIT specification.
mod gen_ {
    wasmtime::component::bindgen!("ml" in "spec/wit/wasi-nn.wit");
}
use gen_::wasi::nn as gen; // Shortcut to the module containing the types we need.

// Export the `types` used in this crate as well as `ML::add_to_linker`.
pub mod types {
    use super::gen;
    pub use gen::graph::{ExecutionTarget, Graph, GraphEncoding};
    pub use gen::inference::GraphExecutionContext;
    pub use gen::tensor::{Tensor, TensorType};
}
pub use gen_::Ml as ML;

impl gen::graph::Host for WasiNnCtx {
    /// Load an opaque sequence of bytes to use for inference.
    fn load(
        &mut self,
        builders: Vec<gen::graph::GraphBuilder>,
        encoding: gen::graph::GraphEncoding,
        target: gen::graph::ExecutionTarget,
    ) -> wasmtime::Result<Result<gen::graph::Graph, gen::errors::Error>> {
        let backend_kind: BackendKind = encoding.try_into()?;
        let graph = if let Some(backend) = self.backends.get_mut(&backend_kind) {
            let slices = builders.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
            backend.load(&slices, target.into())?
        } else {
            return Err(UsageError::InvalidEncoding(encoding.into()).into());
        };
        let graph_id = self.graphs.insert(graph);
        Ok(Ok(graph_id))
    }

    fn load_by_name(
        &mut self,
        _name: String,
    ) -> wasmtime::Result<Result<gen::graph::Graph, gen::errors::Error>> {
        todo!()
    }
}

impl gen::inference::Host for WasiNnCtx {
    /// Create an execution instance of a loaded graph.
    ///
    /// TODO: remove completely?
    fn init_execution_context(
        &mut self,
        graph_id: gen::graph::Graph,
    ) -> wasmtime::Result<Result<gen::inference::GraphExecutionContext, gen::errors::Error>> {
        let exec_context = if let Some(graph) = self.graphs.get_mut(graph_id) {
            graph.init_execution_context()?
        } else {
            return Err(UsageError::InvalidGraphHandle.into());
        };

        let exec_context_id = self.executions.insert(exec_context);
        Ok(Ok(exec_context_id))
    }

    /// Define the inputs to use for inference.
    fn set_input(
        &mut self,
        exec_context_id: gen::inference::GraphExecutionContext,
        index: u32,
        tensor: gen::tensor::Tensor,
    ) -> wasmtime::Result<Result<(), gen::errors::Error>> {
        if let Some(exec_context) = self.executions.get_mut(exec_context_id) {
            exec_context.set_input(index, &tensor)?;
            Ok(Ok(()))
        } else {
            Err(UsageError::InvalidGraphHandle.into())
        }
    }

    /// Compute the inference on the given inputs.
    ///
    /// TODO: refactor to compute(list<tensor>) -> result<list<tensor>, error>
    fn compute(
        &mut self,
        exec_context_id: gen::inference::GraphExecutionContext,
    ) -> wasmtime::Result<Result<(), gen::errors::Error>> {
        if let Some(exec_context) = self.executions.get_mut(exec_context_id) {
            exec_context.compute()?;
            Ok(Ok(()))
        } else {
            Err(UsageError::InvalidExecutionContextHandle.into())
        }
    }

    /// Extract the outputs after inference.
    fn get_output(
        &mut self,
        exec_context_id: gen::inference::GraphExecutionContext,
        index: u32,
    ) -> wasmtime::Result<Result<gen::tensor::TensorData, gen::errors::Error>> {
        if let Some(exec_context) = self.executions.get_mut(exec_context_id) {
            // Read the output bytes. TODO: this involves a hard-coded upper
            // limit on the tensor size that is necessary because there is no
            // way to introspect the graph outputs
            // (https://github.com/WebAssembly/wasi-nn/issues/37).
            let mut destination = vec![0; 1024 * 1024];
            let bytes_read = exec_context.get_output(index, &mut destination)?;
            destination.truncate(bytes_read as usize);
            Ok(Ok(destination))
        } else {
            Err(UsageError::InvalidGraphHandle.into())
        }
    }
}

impl TryFrom<gen::graph::GraphEncoding> for crate::backend::BackendKind {
    type Error = UsageError;
    fn try_from(value: gen::graph::GraphEncoding) -> Result<Self, Self::Error> {
        match value {
            gen::graph::GraphEncoding::Openvino => Ok(crate::backend::BackendKind::OpenVINO),
            _ => Err(UsageError::InvalidEncoding(value.into())),
        }
    }
}
