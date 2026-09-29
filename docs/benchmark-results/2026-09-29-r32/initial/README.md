# Initial R3.2 implementation measurements

This directory retains the implementation before deferred command construction.
Use [the final report](../README.md) for committed-source results. The initial
[source patch](candidate.patch), [environment](environment.json), [build commands](build-commands.json),
[validation](validation.json), [main measurements](measurements.json),
[summary](summary.json), [longer repeats](followup-measurements.json),
[repeat summary](followup-summary.json), and [new-workflow measurements](candidate-only-measurements.json)
remain intact. There are 24 main, 12 longer-repeat and 12 candidate-only runs.

The eager argument tree added +25.27%/+24.01% to the initial inspect/plan control
aggregates. This prompted deferred construction and removal of unrelated argument
clones. Later repeat pairs overlapped our optimization build and have additional
CPU contention; all are retained, but none substitute for final-source evidence.
The final report reruns the complete experiment with the optimized release build.
No threshold result was dropped from the initial records. Initial diagnostics
include the stale-dependency check symptom and a corrected corruption fixture.
