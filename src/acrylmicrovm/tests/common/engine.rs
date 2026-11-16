// Modified by ACRYL, 2026
//! Explicit engine selection for the M2/M3 matrices; product defaults stay unchanged.
use acrylmicrovm::experimental::{ExperimentalFeature, RuntimeBuilder, custom_kernel, native_vmm};
use acrylmicrovm::runtime::options::{BoxOptions, AcrylMicrovmOptions};
use acrylmicrovm::{AcrylMicrovmRuntime, LiteBox};
use acrylmicrovm_shared::{AcrylMicrovmError, AcrylMicrovmResult};

#[derive(Clone, Debug)]
pub struct TestRuntime(AcrylMicrovmRuntime);

fn native() -> AcrylMicrovmResult<bool> {
    match std::env::var("ACRYLMICROVM_TEST_ENGINE").as_deref() {
        Ok("native") => Ok(true),
        Ok("krun") | Err(std::env::VarError::NotPresent) => Ok(false),
        _ => Err(AcrylMicrovmError::InvalidArgument(
            "ACRYLMICROVM_TEST_ENGINE must be native or krun".into(),
        )),
    }
}

impl TestRuntime {
    pub fn new(options: AcrylMicrovmOptions) -> AcrylMicrovmResult<Self> {
        let runtime = if native()? {
            RuntimeBuilder::new(options)
                .enable(ExperimentalFeature::CustomKernel)
                .enable(ExperimentalFeature::NativeVmm)
                .build()?
        } else {
            AcrylMicrovmRuntime::new(options)?
        };
        Ok(Self(runtime))
    }

    pub fn from_runtime(runtime: AcrylMicrovmRuntime) -> Self {
        Self(runtime)
    }

    fn options(mut options: BoxOptions) -> AcrylMicrovmResult<BoxOptions> {
        if native()? {
            let kernel = std::env::var_os("VMM_KERNEL").ok_or_else(|| {
                AcrylMicrovmError::InvalidArgument("native tests require VMM_KERNEL".into())
            })?;
            native_vmm::configure(&mut options);
            custom_kernel::configure(
                &mut options,
                custom_kernel::KernelOptions::new(kernel)
                    .with_format(custom_kernel::KernelFormat::Elf),
            );
        }
        Ok(options)
    }

    pub async fn create(
        &self,
        options: BoxOptions,
        name: Option<String>,
    ) -> AcrylMicrovmResult<LiteBox> {
        self.0.create(Self::options(options)?, name).await
    }

    pub async fn get_or_create(
        &self,
        options: BoxOptions,
        name: Option<String>,
    ) -> AcrylMicrovmResult<(LiteBox, bool)> {
        self.0.get_or_create(Self::options(options)?, name).await
    }
}

impl std::ops::Deref for TestRuntime {
    type Target = AcrylMicrovmRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
