use rvvdk_core::{Capabilities, ExtentKind};

/// Physical operation selected from logical intent. DISCARD retains its current
/// contract; requiring guaranteed zero reads is a separate R2 capability change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    Copy,
    Zero,
    Discard,
    WriteZero,
}

pub(crate) fn select(kind: ExtentKind, capabilities: impl FnOnce() -> Capabilities) -> Operation {
    match kind {
        ExtentKind::Data => Operation::Copy,
        ExtentKind::Zero => zero_operation(capabilities()),
        ExtentKind::Hole => {
            let capabilities = capabilities();
            if capabilities.contains(Capabilities::DISCARD) {
                Operation::Discard
            } else {
                zero_operation(capabilities)
            }
        }
    }
}

fn zero_operation(capabilities: Capabilities) -> Operation {
    if capabilities.contains(Capabilities::WRITE_ZERO) {
        Operation::Zero
    } else {
        Operation::WriteZero
    }
}
