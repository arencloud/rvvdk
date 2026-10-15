# Supplemental SHA-NI CPU investigation (declared after original rounds)

The original SHA-NI repeat wall changes all stay below +3%, but four CPU changes
cross +5%: small proof export +9.04%, bulk proof export +5.62%, bulk owned
transfer +8.68%, and small worker seal +7.64%. Retain these observations. Initial
CPU changes for these operations differ substantially; the small worker's repeat
CPU medians are 0.458 versus 0.493 ms. This is evidence of variation to investigate,
not permission to discard the original result.

Run three rounds of eight adjacent invocation pairs for each affected group:
transfer/disk/small, transfer/disk/bulk, worker/disk/small. Each invocation runs
one fixture pair (both operations). Alternate binary order by pair and alternate
A/A versus A/B control order by round. A/A uses the frozen reference under both
labels; A/B uses the unchanged frozen reference/candidate. Preserve all samples,
round medians and spread. Same CPU 0, filesystem, normal cache and correctness
oracles. No concurrent local builds/tests. These 576 additional observations
supplement rather than replace the original evidence. Persistent consistent
regression requires rejecting/revising the candidate; unexplained variation
remains PERF.0 without a universal no-regression claim.

## Native small-artifact follow-up (declared after the native original rounds)

The native artifact/disk/small owned-export repeat result is +6.67% wall and
+5.65% CPU. All bulk native groups improve. Apply the same three-round/eight-pair
A/A and A/B adjacent-invocation protocol to artifact/disk/small on the runner,
retaining its artifact-admission partner: 192 additional observations. No source
or frozen binary changes. The driver is generalized only to select this case;
the exact earlier SHA-host driver is retained as `control-driver-sha.py`.
