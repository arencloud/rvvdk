use crate::{CopyPlan, DataMover, ExecutionBackend};
use rvvdk_core::{Error, Result, VirtualDisk};

#[cfg(target_os = "linux")]
use crate::io_uring::{NativeExtentPlan, evaluate_compatibility};
#[cfg(target_os = "linux")]
use crate::{ExecutionStrategy, IoUringExecutionOptions};
#[cfg(target_os = "linux")]
use rvvdk_core::{BlockDevice, ExtentKind, RawDisk};
#[cfg(target_os = "linux")]
use rvvdk_platform::LinuxFdBackend;

/// Invocation-scoped checked dispatch state, bound to the validated borrows.
/// This does not claim runtime ring/buffer allocation or snapshot consistency.
pub(crate) struct PreparedExecution<'a, S: ?Sized, D: ?Sized> {
    plan: &'a CopyPlan,
    source: &'a S,
    destination: &'a D,
    executor: PreparedExecutor,
}

pub(crate) enum PreparedExecutor {
    Threaded,
    #[cfg(target_os = "linux")]
    IoUring {
        native_plan: NativeExtentPlan,
        options: IoUringExecutionOptions,
    },
}

impl<'a, S: ?Sized, D: ?Sized> PreparedExecution<'a, S, D> {
    pub(crate) fn parts(&self) -> (&'a CopyPlan, &'a S, &'a D, &PreparedExecutor) {
        (self.plan, self.source, self.destination, &self.executor)
    }
}

impl DataMover {
    pub(crate) fn prepare_portable<'a, S, D>(
        &self,
        plan: &'a CopyPlan,
        source: &'a S,
        destination: &'a D,
    ) -> Result<PreparedExecution<'a, S, D>>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        self.validate_portable_plan(plan, source, destination)?;
        Ok(PreparedExecution {
            plan,
            source,
            destination,
            executor: PreparedExecutor::Threaded,
        })
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn prepare_raw<'a, S, D>(
        &self,
        plan: &'a CopyPlan,
        source: &'a RawDisk<S>,
        destination: &'a RawDisk<D>,
    ) -> Result<PreparedExecution<'a, RawDisk<S>, RawDisk<D>>>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        self.validate_raw_plan(plan, source, destination)?;
        let executor = match plan.backend() {
            ExecutionBackend::Threaded => PreparedExecutor::Threaded,
            ExecutionBackend::IoUring => {
                let execution_options = match self.execution_strategy() {
                    ExecutionStrategy::IoUring(options) | ExecutionStrategy::Auto(options) => {
                        options
                    }

                    ExecutionStrategy::Threaded => {
                        return Err(Error::NativeExecutionNotSelected);
                    }
                };

                let native_extents = plan.extents().to_vec();
                self.native_memory(
                    plan.extent_capacity(),
                    native_extents.capacity(),
                    plan.data_bytes() != 0,
                    plan.extent_count() != 0,
                    execution_options,
                )?
                .check(self.options.memory_budget(), "native plan")?;
                let native_plan = NativeExtentPlan::new(native_extents, plan.logical_bytes())?;

                let source_backend = source.device();

                let destination_backend = destination.device();

                let compatibility = evaluate_compatibility(source_backend, destination_backend);

                if !compatibility.compatible() {
                    return Err(Error::NativeExecutionUnsupported);
                }

                let runtime_alignment = compatibility
                    .alignment()
                    .max(self.options.buffer_alignment());

                if runtime_alignment != plan.alignment() {
                    return Err(Error::InvalidCopyConfiguration(format!(
                        "copy plan alignment mismatch: \
                     plan={}, runtime={runtime_alignment}",
                        plan.alignment(),
                    )));
                }

                PreparedExecutor::IoUring {
                    native_plan,
                    options: execution_options,
                }
            }
        };
        Ok(PreparedExecution {
            plan,
            source,
            destination,
            executor,
        })
    }

    pub(crate) fn require_portable_execution(&self) -> Result<()> {
        crate::ExecutionSelection::portable(self.execution_strategy()).map(|_| ())
    }

    fn validate_portable_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        self.require_portable_execution()?;
        if plan.backend() != ExecutionBackend::Threaded {
            return Err(Error::NativeExecutionUnsupported);
        }
        self.validate_plan(plan, source, destination, || {
            crate::preflight::virtual_pair(source, destination, plan.logical_bytes()).map(|_| ())
        })
    }

    #[cfg(target_os = "linux")]
    fn validate_raw_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<()>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        self.validate_plan(plan, source, destination, || {
            let endpoints = crate::preflight::raw_pair(source, destination, plan.logical_bytes())?;
            if plan.backend() == ExecutionBackend::IoUring
                && plan
                    .extents()
                    .iter()
                    .any(|extent| extent.kind() == ExtentKind::Data)
            {
                endpoints.validate_native_binding()?;
            }
            Ok(())
        })
    }

    /// Validate structural plan invariants before any execution or observation.
    fn validate_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        preflight: impl FnOnce() -> Result<()>,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let source_size = source.size();

        let destination_size = destination.size();

        /*
         * The source geometry must still match the geometry captured by
         * the plan.
         */
        if source_size != plan.logical_bytes() {
            return Err(Error::StaleCopyPlan(format!(
                "copy plan source size mismatch: \
                     plan={}, source={source_size}",
                plan.logical_bytes(),
            )));
        }

        /*
         * The destination must still be large enough for the complete
         * logical source.
         */
        if destination_size < plan.logical_bytes() {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: plan.logical_bytes(),
                size: destination_size,
            });
        }

        preflight()?;

        /*
         * The DataMover configuration used for execution must match the
         * block size captured during planning.
         */
        if plan.block_size() != self.options.block_size() {
            return Err(Error::InvalidCopyConfiguration(format!(
                "copy plan block size mismatch: \
                     plan={}, mover={}",
                plan.block_size(),
                self.options.block_size(),
            )));
        }

        /*
         * Re-read only the source extent metadata.
         *
         * We deliberately do not read or hash Data contents here.
         * CopyPlan represents a structural execution plan, not a snapshot
         * of the source payload.
         */
        // Reject known execution requirements before querying another extent Vec,
        // allocating executor resources, or notifying the observer.
        self.execution_memory(plan)?
            .check(self.options.memory_budget(), "execution")?;
        let current_extents = source.extents(0, source_size)?;
        self.check_extent_memory(
            plan.extent_capacity(),
            current_extents.capacity(),
            "revalidation",
        )?;

        crate::extent_validation::validate_extents(&current_extents, source_size)?;

        let current_fingerprint = crate::plan::extent_fingerprint(&current_extents);

        /*
         * Reject a stale plan when the source extent structure changed
         * between planning and execution.
         *
         * The fingerprint covers:
         *
         *     offset
         *     length
         *     ExtentKind
         */
        if current_fingerprint != plan.extent_fingerprint() {
            return Err(Error::StaleCopyPlan(format!(
                "copy plan extent map changed: \
                     planned={:#018x}, current={:#018x}",
                plan.extent_fingerprint(),
                current_fingerprint,
            )));
        }

        if plan.backend() == ExecutionBackend::Threaded
            && plan.alignment() != self.options.buffer_alignment()
        {
            return Err(Error::InvalidCopyConfiguration(format!(
                "copy plan alignment mismatch: plan={}, mover={}",
                plan.alignment(),
                self.options.buffer_alignment(),
            )));
        }

        Ok(())
    }
}
