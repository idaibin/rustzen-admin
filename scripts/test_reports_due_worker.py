import datetime as dt
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('due_worker', Path(__file__).with_name('verify-reports-due-worker.py'))
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)


class DueWorkerOracleTests(unittest.TestCase):
    def test_whole_minute_due_has_safety_margin_across_minute_and_day_rollover(self):
        for second in range(60):
            for microsecond in (0, 999999):
                now = dt.datetime(2026, 10, 4, 23, 59, second, microsecond, tzinfo=dt.timezone.utc)
                due = worker.due_slot(now)
                self.assertEqual((due.second, due.microsecond), (0, 0))
                self.assertGreaterEqual((due-now).total_seconds(), 30)
                self.assertLessEqual((due-now).total_seconds(), 90)

    def test_terminal_oracle_rejects_wrong_failure_or_missing_timestamps(self):
        valid = {'status': 'failed', 'error': 'target system is disabled', 'startedAt': 'start', 'finishedAt': 'end'}
        worker.assert_terminal(valid)
        for patch in ({'status': 'queued'}, {'status': 'succeeded'}, {'error': 'browser missing'}, {'startedAt': None}, {'finishedAt': None}):
            with self.assertRaises(AssertionError):
                worker.assert_terminal({**valid, **patch})


if __name__ == '__main__':
    unittest.main()
