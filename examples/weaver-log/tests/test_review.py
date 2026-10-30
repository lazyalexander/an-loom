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
        self.root = Path(self.temp.name)
        for name in ('DOMAIN.md', 'materials.md', 'acceptance.sh'):
            (self.root/name).write_text('selected '+name)
        self.settings = dict(sha256={'loom': 'fixture', 'kimi': 'fixture'}, model='fixture-no-model')
        self.plan = dict(inputs=w.review_inputs(self.root), tools=self.settings['sha256'])
        (self.root/'review-plan.json').write_text(json.dumps(self.plan))
        for name in ('prompt.txt', 'generation-plan.json'):
            (self.root/name).write_text('bound '+name)
        self.stored = dict(sha256=w.digest(self.root/'review-plan.json'),
            prompt_sha256=w.digest(self.root/'prompt.txt'),
            generation_plan_sha256=w.digest(self.root/'generation-plan.json'))
        (self.root/'reviewed.json').write_text(json.dumps(self.stored))
        self.patcher = patch.object(w.native, 'settings', return_value=self.settings)
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

    def test_changed_prompt_or_plan_never_dispatches_harness(self):
        for name in ('prompt.txt', 'generation-plan.json'):
            path = self.root/name
            before = path.read_bytes()
            path.write_bytes(before+b' changed')
            with patch.object(w.native, 'generate') as generate:
                with self.assertRaises(RuntimeError):
                    w.generate(SimpleNamespace(output=self.root))
                generate.assert_not_called()
            path.write_bytes(before)

    def test_missing_approval_never_dispatches(self):
        (self.root/'reviewed.json').unlink()
        with patch.object(w.native, 'generate') as generate:
            with self.assertRaises(FileNotFoundError):
                w.generate(SimpleNamespace(output=self.root))
            generate.assert_not_called()

    def test_wrong_approval_cannot_prepare_task(self):
        with patch.object(w.native, 'call') as call:
            with self.assertRaises(RuntimeError):
                w.approve(SimpleNamespace(output=self.root, approve='wrong'))
            call.assert_not_called()
            self.assertFalse((self.root/'approval-intent.json').exists())

    def test_submitted_review_cannot_be_refreshed(self):
        (self.root/'approval-intent.json').write_text('{}')
        with self.assertRaises(RuntimeError):
            w.review(SimpleNamespace(output=self.root))

    def test_changed_tool_identity_is_not_reused(self):
        self.settings['sha256'] = {'loom': 'different', 'kimi': 'fixture'}
        with self.assertRaises(RuntimeError):
            w.checked(self.root, approved=True)


if __name__ == '__main__':
    unittest.main()
