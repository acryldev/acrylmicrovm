// Modified by ACRYL, 2026
//! Generic task trait for pipeline execution.

use acrylmicrovm_shared::errors::AcrylMicrovmResult;
use async_trait::async_trait;

/// Trait for tasks that can be executed in a pipeline.
///
/// Implement this trait to define custom task types for your pipeline.
/// Tasks run with a shared context, which is cloned per task.
#[allow(
    clippy::double_must_use,
    reason = "async_trait generates a must-use boxed future"
)]
#[async_trait]
pub trait PipelineTask<Ctx>: Send + Sync {
    /// Execute the task with the shared pipeline context.
    async fn run(self: Box<Self>, ctx: Ctx) -> AcrylMicrovmResult<()>;

    /// Get human-readable task name for logging.
    fn name(&self) -> &str;
}

pub type BoxedTask<Ctx> = Box<dyn PipelineTask<Ctx>>;
