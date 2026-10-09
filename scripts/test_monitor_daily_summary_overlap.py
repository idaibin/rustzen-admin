"""Pure overlap-oracle tests: no HTTP requests, sleeps or cohort execution."""
import unittest
from monitor_daily_summary_overlap import maximum_overlap


class OverlapOracleTests(unittest.TestCase):
    def test_four_overlapping_intervals(self):
        self.assertEqual(maximum_overlap([(0, 4), (1, 5), (2, 6), (3, 7)]), 4)

    def test_touching_and_sequential_intervals_are_not_parallel(self):
        self.assertEqual(maximum_overlap([(0, 1), (1, 2), (2, 3), (3, 4)]), 1)
        self.assertEqual(maximum_overlap([(1, 1)]), 0)

    def test_invalid_interval_is_rejected(self):
        with self.assertRaises(ValueError):
            maximum_overlap([(2, 1)])


if __name__ == '__main__':
    unittest.main()
