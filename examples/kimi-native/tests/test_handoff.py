"""Credential-free proposal parsing and no-retry admission tests."""
import argparse
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('handoff', Path(__file__).parents[1] / 'qualify.py')
handoff = importlib.util.module_from_spec(spec)
spec.loader.exec_module(handoff)


class HandoffTests(unittest.TestCase):
    def test_exact_response_and_qualified_bullet_rendering(self):
        value = {'schema': 'an-loom-kit-candidate-v1', 'path': 'kit/transform.sh', 'content': 'exit 0\n'}
        raw = json.dumps(value).encode()
        self.assertEqual(handoff.parse_candidate(raw), value)
        self.assertEqual(handoff.parse_candidate('• '.encode() + raw), value)
        self.assertEqual(handoff.parse_candidate(b'```json\n' + raw + b'\n```'), value)
        self.assertEqual(handoff.parse_candidate('• ```json\n  '.encode() + raw + b'\n  ```'), value)
        for bad in [b'prose ' + raw, raw + raw, b'```json\n' + raw + b'\n``` prose', b'x' * 65537]:
            with self.assertRaises(ValueError):
                handoff.parse_candidate(bad)
        for change in [{'path': 'silk/main.sh'}, {'path': '../escape'}, {'content': ''}, {'content': []}, {'execute': True}]:
            with self.assertRaises(ValueError):
                handoff.parse_candidate(json.dumps(dict(value, **change)).encode())

    def test_prior_generation_intent_never_starts_another_client(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'prompt.txt').write_text('public task')
            handoff.write(root / 'generation-plan.json', {'prompt_sha256': handoff.digest(root / 'prompt.txt')})
            handoff.write(root / 'generation-intent.json', {'outcome': 'unknown'})
            args = argparse.Namespace(output=root, approve=handoff.digest(root / 'generation-plan.json'))
            with patch.object(handoff, 'settings', return_value={}), patch.object(handoff.subprocess, 'run') as run:
                with self.assertRaises(FileExistsError):
                    handoff.generate(args)
                run.assert_not_called()
            self.assertFalse((root / 'client').exists())

class ReceptionTests(unittest.TestCase):
    def test_unknown_generation_cannot_be_extracted(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            handoff.write(root / 'generation-result.json', {'status': 'unknown'})
            with patch.object(handoff, 'settings', return_value={}):
                with self.assertRaises(RuntimeError):
                    handoff.extract(argparse.Namespace(output=root))
            self.assertFalse((root / 'candidate.sh').exists())

    def test_unreviewed_digest_cannot_reach_loom(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            value = {'schema': 'an-loom-kit-candidate-v1', 'path': 'kit/transform.sh', 'content': 'exit 0\n'}
            handoff.write(root / 'generation-result.json', {'status': 'completed'})
            handoff.write(root / 'response.txt', value)
            (root / 'candidate.sh').write_text(value['content'])
            args = argparse.Namespace(output=root, source_sha256='0' * 64)
            with patch.object(handoff, 'settings', return_value={'loom': 'not-invoked'}), patch.object(handoff.subprocess, 'run') as run:
                with self.assertRaises(RuntimeError):
                    handoff.receive(args)
                run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
