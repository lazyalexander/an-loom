"""Credential-free review gates; approvals below are synthetic test fixtures."""
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('weaver', Path(__file__).resolve().parents[1]/'weaver.py')
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class ReviewGate(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        for name in ('DOMAIN.md', 'materials.md', 'acceptance.sh'):
            (self.root/name).write_text('selected '+name)
        self.settings = dict(loom='not-invoked', loom_sha256='fixture')
        self.plan = dict(schema='weaver-log-review-v2', inputs=w.review_inputs(self.root),
                         loom_sha256=self.settings['loom_sha256'])
        (self.root/'review-plan.json').write_text(json.dumps(self.plan))
        (self.root/'task/authority').mkdir(parents=True)
        for name in ('prompt.txt', 'task/authority/task.json', 'candidate.sh'):
            (self.root/name).write_text('bound '+name)
        self.stored = dict(sha256=w.digest(self.root/'review-plan.json'),
            prompt_sha256=w.digest(self.root/'prompt.txt'),
            task_sha256=w.digest(self.root/'task/authority/task.json'))
        (self.root/'reviewed.json').write_text(json.dumps(self.stored))
        self.patcher = patch.object(w, 'settings', return_value=self.settings)
        self.patcher.start()
        self.addCleanup(self.patcher.stop)

    def test_selected_bytes_cannot_change_after_approval(self):
        w.checked(self.root, approved=True)
        for name in ('DOMAIN.md', 'materials.md', 'acceptance.sh'):
            path = self.root/name
            before = path.read_bytes()
            path.write_bytes(before+b' changed')
            with self.assertRaises(RuntimeError):
                w.checked(self.root, approved=True)
            path.write_bytes(before)

    def receive(self):
        w.receive(SimpleNamespace(output=self.root, source_sha256=w.digest(self.root/'candidate.sh')))

    def test_changed_prompt_or_task_cannot_receive_candidate(self):
        for name in ('prompt.txt', 'task/authority/task.json'):
            path = self.root/name
            before = path.read_bytes()
            path.write_bytes(before+b' changed')
            with patch.object(w, 'call') as call:
                with self.assertRaises(RuntimeError):
                    self.receive()
                call.assert_not_called()
            path.write_bytes(before)

    def test_missing_approval_cannot_receive_candidate(self):
        (self.root/'reviewed.json').unlink()
        with patch.object(w, 'call') as call:
            with self.assertRaises(FileNotFoundError):
                self.receive()
            call.assert_not_called()

    def test_wrong_approval_cannot_prepare_task(self):
        with patch.object(w, 'call') as call:
            with self.assertRaises(RuntimeError):
                w.approve(SimpleNamespace(output=self.root, approve='wrong'))
            call.assert_not_called()
            self.assertFalse((self.root/'approval-intent.json').exists())

    def test_submitted_review_cannot_be_refreshed(self):
        (self.root/'approval-intent.json').write_text('{}')
        with self.assertRaises(RuntimeError):
            w.review(SimpleNamespace(output=self.root))

    def test_changed_tool_identity_is_not_reused(self):
        self.settings['loom_sha256'] = 'different'
        with self.assertRaises(RuntimeError):
            w.checked(self.root, approved=True)

    def test_unreviewed_candidate_never_reaches_loom(self):
        with patch.object(w, 'call') as call:
            with self.assertRaises(RuntimeError):
                w.receive(SimpleNamespace(output=self.root, source_sha256='wrong'))
            call.assert_not_called()

    def test_external_bytes_need_no_provider_success_record(self):
        with patch.object(w, 'call') as call:
            self.receive()
            call.assert_called_once_with('not-invoked', 'receive-task', self.root/'task',
                                         'kit/transform.sh', self.root/'candidate.sh')
        self.assertFalse((self.root/'generation-result.json').exists())

    def test_legacy_review_is_not_silently_reinterpreted(self):
        self.plan['schema'] = 'weaver-log-review-v1'
        (self.root/'review-plan.json').write_text(json.dumps(self.plan))
        with self.assertRaises(RuntimeError):
            w.checked(self.root)


if __name__ == '__main__':
    unittest.main()
