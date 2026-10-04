"""Fast verifier fixture/oracle tests; these do not replace its Rust runtime gate."""
import copy
import datetime as dt
import importlib.util
from pathlib import Path
import sqlite3
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('daily_runtime', Path(__file__).with_name('verify-monitor-daily-summary-runtime.py'))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
ROOT = Path(__file__).resolve().parents[1]


class DailyRuntimeTests(unittest.TestCase):
    def test_fixture_only_seeds_inputs_and_preserves_registration_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'monitor.db'
            with sqlite3.connect(path) as db:
                db.executescript((ROOT / 'apps/monitor/migrations/0001_init.sql').read_text())
            MODULE.seed_database(path, dt.date(2026, 10, 3))
            with sqlite3.connect(path) as db:
                self.assertEqual(db.execute('SELECT COUNT(*) FROM node_daily_summaries').fetchone()[0], 0)
                self.assertEqual(db.execute('SELECT COUNT(*) FROM resource_samples').fetchone()[0], 2)
                self.assertEqual(db.execute('SELECT COUNT(*) FROM disk_samples').fetchone()[0], 4)
                self.assertEqual(db.execute("SELECT created_at FROM monitor_nodes WHERE node_id='future'").fetchone()[0], '2026-10-04T00:00:00+00:00')
                self.assertEqual(db.execute('PRAGMA foreign_key_check').fetchall(), [])

    def test_fixture_refuses_preexisting_summary_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'monitor.db'
            with sqlite3.connect(path) as db:
                db.executescript((ROOT / 'apps/monitor/migrations/0001_init.sql').read_text())
                db.execute("INSERT INTO node_daily_summaries(node_id,summary_date,sample_count,disk_summary_json) VALUES('preexisting','2026-10-03',0,'{}')")
            with self.assertRaises(AssertionError):
                MODULE.seed_database(path, dt.date(2026, 10, 3))

    def test_oracle_accepts_expected_values_and_rejects_regressions(self):
        empty = {'nodeId': 'empty', 'date': '2026-10-03', 'sampleCount': 0, 'coveragePercent': 0,
                 'cpu': {'min': None, 'avg': None, 'max': None},
                 'memory': {'min': None, 'avg': None, 'max': None},
                 'diskSummary': {}, 'offlineSeconds': 0, 'incidentCount': 0}
        sampled = {'nodeId': 'sampled', 'date': '2026-10-03', 'sampleCount': 2, 'coveragePercent': 2 / 2880 * 100,
                   'cpu': {'min': 10.0, 'avg': 20.0, 'max': 30.0},
                   'memory': {'min': 30.0, 'avg': 50.0, 'max': 70.0},
                   'diskSummary': {'/': {'min': 20.0, 'avg': 30.0, 'max': 40.0}, '/data': {'min': 40.0, 'avg': 50.0, 'max': 60.0}},
                   'offlineSeconds': 180, 'incidentCount': 2}
        MODULE.verify_rows([empty, sampled], dt.date(2026, 10, 3))
        for key, value in [('sampleCount', 1), ('offlineSeconds', 240), ('coveragePercent', 100),
                           ('incidentCount', 1), ('cpu', {}), ('memory', {}), ('diskSummary', {})]:
            with self.subTest(field=key):
                wrong = copy.deepcopy(sampled)
                wrong[key] = value
                with self.assertRaises(AssertionError):
                    MODULE.verify_rows([empty, wrong], dt.date(2026, 10, 3))

    def test_oracle_rejects_empty_and_duplicate_results(self):
        for rows in [[], [{'nodeId': 'empty'}] * 2]:
            with self.assertRaises(AssertionError):
                MODULE.verify_rows(rows, dt.date(2026, 10, 3))


if __name__ == '__main__':
    unittest.main()
