use rvvdk_core::{Capabilities, ExtentKind};

/// Physical operation selected from logical intent. Hole output must read zero;
/// ordinary discard is insufficient without its explicit zero-read guarantee.
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
            if capabilities.contains(Capabilities::DISCARD | Capabilities::DISCARD_ZEROES) {
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
