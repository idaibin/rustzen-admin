"""Mocked harness fail-closed tests, separate from the real HTTP observation."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import monitor_daily_summary_load as load


class LoadHarnessTests(unittest.TestCase):
    def run_fixture(self, request, usage=None):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(load, 'process_usage', side_effect=usage or (lambda _: {'rssBytes': 1, 'cpuSeconds': 0})), patch.object(load, 'GLOBAL_RPS', 100000):
                try:
                    load.verify_read_observation(request, 'fixture-token', [object(), object()],
                                                 [{'page': 1, 'body': {'page': 1}}, {'page': 2, 'body': {'page': 2}}], Path(directory))
                except AssertionError:
                    pass
            return json.loads((Path(directory) / 'load-observation.json').read_text())

    def test_success_stops_at_exact_request_cap(self):
        calls = []
        def request(path, _):
            calls.append(path)
            return 200, {'page': 1 if 'current=1&' in path else 2}
        result = self.run_fixture(request)
        self.assertEqual(len(calls), 100)
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['sampleCount'], 100)
        self.assertLessEqual(result['maxObservedInFlight'], 4)

    def test_wrong_payload_stops_dispatch_and_preserves_failure(self):
        calls = []
        def request(path, _):
            calls.append(path)
            return 200, {'wrong': 'page'}
        result = self.run_fixture(request)
        self.assertEqual(result['status'], 'failed')
        self.assertGreater(result['errorCount'], 0)
        self.assertLessEqual(len(calls), 4)

    def test_resource_ceiling_stops_before_request(self):
        calls = []
        result = self.run_fixture(lambda *args: calls.append(args),
                                  lambda _: {'rssBytes': load.RSS_LIMIT, 'cpuSeconds': 0})
        self.assertEqual(calls, [])
        self.assertEqual(result['status'], 'failed')
        self.assertTrue(any('RSS resource ceiling' in error for error in result['failures']))


if __name__ == '__main__':
    unittest.main()
