import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('gateway_read', Path(__file__).with_name('verify-reports-gateway-read.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def fixture():
    row = {'id':'owned-run','status':'failed','error':'target system is disabled'}
    observed = {'run':row,'occurrence':{'runId':row['id']}}
    return {'status':'passed','requestCount':8,'processRestarts':1,
            'unexpectedTargetOrNotificationRequests':[],
            'observations':[copy.deepcopy(observed),copy.deepcopy(observed)]}


class GatewayReadOracleTests(unittest.TestCase):
    def test_accepts_only_matching_retained_failed_run(self):
        self.assertEqual(module.expected_run(fixture())['id'],'owned-run')

    def test_rejects_failed_receipt_wrong_link_and_drifted_restart(self):
        cases=[]
        bad=fixture();bad['status']='failed';cases.append(bad)
        bad=fixture();bad['unexpectedTargetOrNotificationRequests']=['unexpected'];cases.append(bad)
        bad=fixture();bad['observations'][1]['occurrence']['runId']='other';cases.append(bad)
        bad=fixture();bad['observations'][1]['run']['error']='different';cases.append(bad)
        for bad in cases:
            with self.assertRaises(AssertionError):
                module.expected_run(bad)


if __name__=='__main__':
    unittest.main()
