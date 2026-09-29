# VMDK backing acquisition and resolution (R4.2)

R4.2 turns validated descriptor metadata into retained read-only sources. It does
not implement logical VMDK reads or CLI VMDK support; those follow in R4.3 and a
separate CLI integration step. [Descriptor syntax](vmdk-descriptor.md) is unchanged.

## API and ownership

`DescriptorText::read_from(reader, limits)` owns descriptor bytes, enforces the
acquisition limit without trusting file length, and validates syntax. It reads at
most the allowed bytes plus one stack probe byte, handles short/interrupted reads,
and propagates I/O errors. Its 4 KiB stack chunk avoids a size-hint allocation.
`parse()` borrows its buffer and reapplies the same limits; no self-referential
struct or unsafe lifetime extension is used. Parsing during acquisition and later
parsing for resolution are separate costs. Generic readers must honor Rust's Read
contract; this does not impose a deadline on blocking or endlessly interrupted I/O.

`BackingResolver: Send + Sync` maps an opaque reference to an owned
`Arc<dyn BlockDevice>`. Custom implementations supply namespace policy, authorization
and stable source behavior. They can use memory, remote objects or local files;
portable resolution does not impose pathname syntax. Sources must report live
capacity/access through `copy_endpoint`. Existing endpoint identities are forwarded;
`None` is explicitly unknown, never evidence of non-aliasing or snapshot identity.

`ResolvedDescriptor::resolve` collects requirements before calling the resolver:

1. Bound the number of resolved extents (default 1,024).
2. Count distinct exact reference strings (default at most 128 backing objects).
3. Compute each reference's maximum required physical end from validated extents.
4. Open each distinct reference once, retaining its owned source; check live READ
   capability and length. ZERO needs no backing source.

Count-limit failures call no resolver. Partial failure drops already acquired
sources; a custom resolver's external effects remain its responsibility. Exact
reference reuse shares one handle. Different names, including hard links, count
separately; identities expose known aliases without silently coalescing them.
No disk-capacity-sized allocation or backing-content read occurs during resolution.
Vectors/hash tables have normal bounded growth overhead, not an exact byte budget.
The caller's chosen limits and custom resolver's internal resources are separate.

The result owns layout metadata and sources, so input text and resolver can be
dropped. Informational DDB strings are available on the parsed descriptor, not
copied into the resolved layout. Resolved sources expose physical read-only methods,
initial/live endpoint facts and required length. They do not expose a writable
source handle or claim a native RAW endpoint for the virtual disk.

`revalidate()` inspects the retained objects without reopening names. It rejects
loss of READ, insufficient live length, or a changed/lost previously known identity.
Growth or truncation of an unused tail is allowed. An initially unknown identity
remains unqualified. Initial validation and revalidation are observations, not a
snapshot, multi-file atomic operation, lease, or protection against later content
writes. The future logical reader must revalidate before execution and handle
short reads/errors; it must not silently treat truncation as zeros.

## Linux local policy

`LocalResolver::open_descriptor(path, limits)` treats the caller-selected descriptor
path/parent as trusted configuration. It opens and retains the parent directory,
then loads the descriptor basename with confined lookup. Extent names are relative
to that retained descriptor directory, not the process working directory.
`from_directory(File)` lets a caller explicitly supply the same trusted anchor.
It checks that the handle represents a directory. Trusted parent selection may
follow symlinks; this is distinct from untrusted references beneath that anchor.

Local references must be nonempty UTF-8, at most 4,096 bytes, and composed of
nonempty slash-separated components. Absolute paths, `.`, `..`, repeated/trailing
slashes, backslashes, colons (including drive/URI syntax) and controls reject.
Spaces and Unicode filenames remain supported. The parser may accept names that
this stricter local policy rejects. No permissive policy option is offered yet.

The lookup uses Linux `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS`
(which also excludes magic links), and `RESOLVE_NO_XDEV`. It rejects symlinks in
any component and mount crossings, including bind mounts. A directory FD plus
kernel resolution restrictions avoids a check-then-open pathname race. Unsupported
kernels, seccomp denial, or transient lookup failures propagate; there is no weaker
fallback. These flags are described by the [openat2 manual](https://man7.org/linux/man-pages/man2/openat2.2.html).

Lookup first obtains `O_PATH | O_CLOEXEC`, then requires a regular file. This avoids
opening FIFO/device contents before discovering their type. A read-only I/O handle
is opened through our retained `/proc/self/fd/N` entry, with device/inode checked
against the pinned object. The untrusted pathname is never reopened. Retained
handles survive rename/unlink; this follows Linux's [open/O_PATH semantics](https://man7.org/linux/man-pages/man2/open.2.html).
A working, trusted procfs is required. Local adoption uses the existing buffered
`LocalFileBlockDevice`, retaining its cooperative I/O admission and live endpoint
inspection. Both transient and retained local FDs are close-on-exec. Resolution
opens read-only even when the descriptor declares RW.

Confinement is a lookup policy, not provenance of file contents. Hard links already
inside the selected tree are authorized namespace entries; they can alias objects
with names elsewhere. Directory renames after anchoring do not revoke access to
that directory. Concurrent in-place writes remain possible. An attacker replacing
a regular entry before lookup can supply another regular file within the allowed
tree; checks do not authenticate its contents. Kernel/filesystem/procfs trust and
initial anchor selection belong to the caller/environment. Other platforms retain
the portable loader/resolver APIs but do not export this Linux local resolver.

## Example

```rust,ignore
use rvvdk_vmdk::{Limits, LocalResolver, ResolutionLimits, ResolvedDescriptor};

let (text, resolver) = LocalResolver::open_descriptor("images/disk.vmdk", Limits::default())?;
let parsed = text.parse()?;
let sources = ResolvedDescriptor::resolve(&parsed, &resolver, ResolutionLimits::default())?;
sources.revalidate()?;
// R4.3 will map logical reads over these retained FLAT/ZERO sources.
```

## Validation and next step

Tests cover generic opaque references, repeated-reference ownership, zero-only
layouts, pre-open resource limits, partial-failure cleanup, access/length/identity
changes, bounded acquisition and I/O errors. Linux fixtures cover parent-relative
nested files, unsafe paths, final/intermediate symlinks, proc magic links, mount
crossings, FIFO/socket/directory rejection, pinned directory/entry replacement,
hard-link identities and live truncation. A deterministic unit test replaces the
name between O_PATH pinning and I/O reopen and verifies the original bytes and
read-only/CLOEXEC flags. Linux tests also run on the recorded Btrfs storage.

[Performance evidence](benchmark-results/2026-09-29-r42/README.md) includes matched
parser/RAW controls, one/32-source resolution, 1,024 references sharing one source,
bounded loading, raw results and SVG/PNG plots. No read-mapping/reference-decoder,
ESXi, cold-storage, race-stress or hostile-kernel qualification is claimed.

R4.3 adds read-only FLAT/ZERO `VirtualDisk` mapping, cross-extent reads and byte
comparison against a recorded reference tool. Preserve rejection of unsupported
formats and keep native RAW acceleration separate. No ESXi trial is needed yet.
