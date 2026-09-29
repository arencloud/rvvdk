# Initial R2.4 candidate — inline request diagnostic

Preserved exploratory measurements from the first implementation. This source
used an inline NativeRequestIssue inside core Error, increasing its x86_64 size
from 40 to 48 bytes. Review changed that cold diagnostic to a Box before final
acceptance; the final candidate is measured separately in the parent directory.

This archive has its own candidate.patch, environment/source/binary hashes,
54 main runs, six candidate-only tail runs, and 18 longer-repeat runs. Never
combine these samples with the final candidate or attribute timing changes
between revisions solely to boxing. No causal boxing speedup claim is made.

The initial mixed direct-source aggregate was +20.35%, but a longer repeat was
−10.80%; direct-destination had a +82.09% main pair and −21.70% repeat pair.
Dense copying included +12.22% main and −9.88% repeat pairs. These opposing
results preserve evidence of substantial variation and unresolved attribution.

The tail baseline smoke fails the harness's expected Threaded selection; it
still selects IoUring. This is not proof that the filesystem would reject every
unaligned native copy. Candidate smoke/copy checks pass with explicit fallback.

The initial-test-build.txt diagnostic records the test-only missing libc import
before adding the Linux dev dependency. Final validation for this revision is
in validation.json; final accepted source validation is in the parent directory.
