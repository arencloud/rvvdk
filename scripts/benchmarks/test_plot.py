"""Checks for misleading or malformed benchmark evidence before rendering."""
import copy
import unittest

from plot import compare, samples


def run(tag, variant, pair, value):
    return {'tag': tag, 'variant': variant, 'pair': pair, 'benchmark': 'fixture',
            'criterion': {'sample.json': {'times': [value * 2, value * 3], 'iters': [2, 3]},
                          'estimates.json': {'median': {'point_estimate': value}}}}


def evidence():
    # Deliberately scrambled file order and nonconsecutive pair IDs.
    runs = [run('c7', 'candidate', 7, 36), run('b2', 'baseline', 2, 10),
            run('c2', 'candidate', 2, 20), run('b7', 'baseline', 7, 30)]
    summary = [{'benchmark': 'fixture', 'baseline_ns': 20, 'candidate_ns': 28,
                'change_percent': 40, 'paired_changes_percent': [100, 20]}]
    return runs, summary


class EvidenceTests(unittest.TestCase):
    def test_pair_identity_and_ratio_of_medians_not_median_of_ratios(self):
        runs, summary = evidence()
        result = compare(runs, summary)[0]
        self.assertEqual(result['pair_ids'], [2, 7])
        self.assertAlmostEqual(result['change_percent'], 40)
        self.assertAlmostEqual(result['paired_changes_percent'][0], 100)
        self.assertAlmostEqual(result['paired_changes_percent'][1], 20)

    def test_sample_normalization_keeps_outliers(self):
        value = run('sample', 'candidate', 1, 11)
        value['criterion']['sample.json'] = {'times': [20, 33, 4000], 'iters': [2, 3, 4]}
        self.assertEqual(samples(value), [10, 11, 1000])

    def test_incomplete_or_duplicate_pairs_are_rejected(self):
        runs, summary = evidence()
        for bad in [runs[:-1], runs + [run('another', 'candidate', 7, 36)]]:
            with self.subTest(runs=bad), self.assertRaises(ValueError):
                compare(bad, summary)

    def test_raw_sample_and_summary_disagreement_are_rejected(self):
        runs, summary = evidence()
        bad = copy.deepcopy(runs)
        bad[0]['criterion']['sample.json']['times'][0] *= 2
        with self.assertRaises(ValueError):
            compare(bad, summary)
        summary[0]['change_percent'] = 60
        with self.assertRaises(ValueError):
            compare(runs, summary)

    def test_invalid_samples_are_rejected(self):
        for times, iters in [([], []), ([1], [0]), ([float('nan')], [1]),
                             ([1], [float('inf')]), ([1, 2], [1]), ([-1], [1])]:
            with self.subTest(times=times, iters=iters):
                value = run('invalid', 'candidate', 1, 1)
                value['criterion']['sample.json'] = {'times': times, 'iters': iters}
                with self.assertRaises(ValueError):
                    samples(value)


if __name__ == '__main__':
    unittest.main()
