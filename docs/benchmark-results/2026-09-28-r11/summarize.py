"""Recompute medians of run means and paired changes from saved Criterion data."""
from pathlib import Path
import json
import statistics

root = Path(__file__).resolve().parent
phases = {}
for path in (root / "samples").glob("*/**/new/estimates.json"):
    label = path.relative_to(root / "samples").parts[0]
    pair, variant, bench = label.split("-", 2)
    phase = "initial" if pair.startswith("pair") else "followup"
    name = "/".join(path.relative_to(root / "samples" / label).parts[:-2])
    mean = json.loads(path.read_text())["mean"]["point_estimate"]
    phases.setdefault(phase, {}).setdefault(name, {}).setdefault(variant, {})[pair] = mean
summary = {}
for phase, rows in sorted(phases.items()):
    summary[phase] = {}
    for name, variants in sorted(rows.items()):
        result = {
            variant: {"runs_ns": runs, "median_ns": statistics.median(runs.values())}
            for variant, runs in variants.items()
        }
        if "baseline" in variants and "candidate" in variants:
            pairs = {
                pair: 100 * (variants["candidate"][pair] / baseline - 1)
                for pair, baseline in variants["baseline"].items()
                if pair in variants["candidate"]
            }
            result["paired_change_percent"] = pairs
            if pairs:
                result["median_paired_change_percent"] = statistics.median(pairs.values())
        summary[phase][name] = result
(root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
for phase, rows in summary.items():
    for name, result in rows.items():
        values = ", ".join(
            f"{variant}={value['median_ns'] / 1000:.2f} us"
            for variant, value in result.items()
            if variant in ["baseline", "candidate"]
        )
        print(phase, name, values, result.get("paired_change_percent", {}))
