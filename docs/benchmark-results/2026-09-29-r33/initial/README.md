# Initial R3.3 experiment

The [source patch](candidate.patch), [environment](environment.json),
[build commands](build-commands.json), [48 main records](measurements.json),
[summary](summary.json) and [12 new-mode records](candidate-only-measurements.json)
retain the implementation before fixing a non-Linux enum reference. None of its
main adverse aggregates/pairs crossed +5%; no initial longer repeats were triggered.

The initial portable check failed. The platform guard was fixed, final-source
validation rerun, and new independent release binaries produced. Some hashes
changed despite the intended Linux-equivalent guard; the final report therefore
uses a full fresh experiment instead of reassigning these samples. Final-source
validation logs copied here during the transition are not proof of the original
source's portability; [initial-validation.json](initial-validation.json) and
[initial-portable.txt](initial-portable.txt) retain the original failure.
[Artifact comparison](final-artifact-comparison.json) preserves exact hashes.
Use the [final report](../README.md) for final results and acceptance.
