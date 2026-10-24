"""Negative evidence gates; fixtures do not substitute for native GUI acceptance."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import report


class CaptureGate(unittest.TestCase):
    def capture(self, stopped="requested", alive=0, observed=True, unknown=False):
        root = Path(self.directory.name)
        events = [{"t": 1, "name": "measurement.stopped", "reason": stopped}]
        if observed:
            events.insert(0, {"t": .5, "name": "editor.process.absent"})
        (root / "events.jsonl").write_text("\n".join(map(json.dumps, events)))
        samples = [{"t": t, "role": "editor", "alive": count, "unknown": unknown} for t, count in [(0, int(observed)), (1, alive)]]
        (root / "samples.jsonl").write_text("\n".join(map(json.dumps, samples)))
        (root / "acceptance.json").write_text(json.dumps({"schema": "an-native-office-acceptance-v1"}))
        return root

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)

    def reject(self, root, message):
        output = root / "report"
        with self.assertRaisesRegex(ValueError, message):
            report.run(root, output)
        self.assertFalse(output.exists(), "Failed qualification must not leave success charts")

    def test_expired_capture_is_not_success(self):
        self.reject(self.capture(stopped="duration-limit"), "expired")

    def test_no_editor_is_not_native_acceptance(self):
        self.reject(self.capture(observed=False), "No actual")

    def test_access_denied_is_not_unloaded(self):
        self.reject(self.capture(unknown=True), "unload not confirmed")

    def test_old_exit_does_not_prove_restarted_editor_unloaded(self):
        self.reject(self.capture(alive=1), "unload not confirmed")


if __name__ == "__main__":
    unittest.main()
