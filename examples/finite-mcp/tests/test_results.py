"""Check durable MCP results without a client, model or native execution."""
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('qualification', Path(__file__).resolve().parents[1] / 'qualify.py')
q = importlib.util.module_from_spec(spec)
spec.loader.exec_module(q)


class HostResults(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        source = self.root / 'sort-kit/kit/main.rs'
        source.parent.mkdir(parents=True)
        source.write_text('selected public source')
        self.calls = self.root / 'development-session/calls'
        self.record('inspect-1', dict(sha256=q.digest(source)))
        self.record('sample-1', dict(status='succeeded', reaped=True,
                    publication=dict(status='committed'), artifact=dict(revision='fixture'), text='alpha\nbeta\n'))
        self.record('denied-1', dict(status='refused-or-uncertain', error='kit capability is not granted'))
        candidate = self.calls / 'edit-1/candidate.rs'
        candidate.parent.mkdir(parents=True)
        candidate.write_text('inert candidate')
        self.record('edit-1', dict(executed=False, accepted=False, sha256=q.digest(candidate)))
        self.args = SimpleNamespace(output=self.root)
        settings = {}
        for name in ['loom', 'host', 'bridge', 'toolchain']:
            path = self.root / name
            path.write_text('selected fixture ' + name)
            settings[name] = str(path)
        settings['sha256'] = {name: q.digest(path) for name, path in settings.items()}
        (self.root / 'settings.local.json').write_text(json.dumps(settings))

    def record(self, name, value):
        path = self.calls / name / 'result.json'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value))

    def test_requires_committed_artifact_not_just_success_text(self):
        q.check(self.args)
        self.record('sample-1', dict(status='succeeded', reaped=True,
                    publication=dict(status='not-attempted'), text='alpha\nbeta\n'))
        with self.assertRaises(RuntimeError):
            q.check(self.args)

    def test_uncertain_failure_is_not_a_verified_denial(self):
        self.record('denied-1', dict(status='refused-or-uncertain', error='storage unavailable'))
        with self.assertRaises(RuntimeError):
            q.check(self.args)

    def test_changed_candidate_cannot_pass(self):
        (self.calls / 'edit-1/candidate.rs').write_text('changed candidate')
        with self.assertRaises(RuntimeError):
            q.check(self.args)

    def test_unverified_run_cannot_build_a_candidate(self):
        self.record('denied-1', dict(status='refused-or-uncertain', error='storage unavailable'))
        with patch.object(q, 'build') as build:
            with self.assertRaises(RuntimeError):
                q.accept(self.args)
            build.assert_not_called()

    def test_legacy_client_fixture_cannot_be_reinterpreted(self):
        path = self.root / 'settings.local.json'
        settings = json.loads(path.read_text())
        settings['kimi'] = '/historical/client'
        settings['sha256']['instructions'] = 'historical-profile-digest'
        path.write_text(json.dumps(settings))
        with patch.object(q, 'build') as build:
            with self.assertRaisesRegex(RuntimeError, 'legacy client-bound'):
                q.accept(self.args)
            build.assert_not_called()


if __name__ == '__main__':
    unittest.main()
