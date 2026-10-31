"""Domain tests run standalone; no Loom, host, models or GUI are required."""
from pathlib import Path
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parents[1]


class Accumulator(unittest.TestCase):
    def execute(self, work, data, state, success=True):
        source, checkpoint = work / "batch.log", work / "checkpoint.txt"
        source.write_text(data)
        checkpoint.write_text(state)
        result = subprocess.run(["/bin/bash", str(HERE / "kit/accumulate.sh"), str(source), str(checkpoint)],
                                cwd=work, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode == 0, success, result.stderr)

    def test_accumulates_only_the_supplied_checkpoint(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.execute(root, (HERE / "sample.txt").read_text(), (HERE / "initial-state.txt").read_text())
            self.assertEqual((root / "report.txt").read_text(), "total=3\nINFO=2\nWARN=0\nERROR=1\n")
            next_state = (root / "state.txt").read_text()
            other = root / "other"
            other.mkdir()
            self.execute(other, "WARN changed\n", next_state)
            self.assertEqual((other / "state.txt").read_text(), "total=4\nINFO=2\nWARN=1\nERROR=1\n")

    def test_invalid_input_never_creates_successful_outputs(self):
        cases = [("UNKNOWN bad\n", (HERE / "initial-state.txt").read_text()),
                 ("INFO valid\n", "total=1\nINFO=0\nWARN=0\nERROR=0\n"),
                 ("INFO valid\n", "total=1000000\nINFO=1000000\nWARN=0\nERROR=0\n"),
                 ("INFO valid\n", "total=0\nINFO=0\nWARN=0\nERROR=0\nINFO=0\n")]
        for data, state in cases:
            with self.subTest(state=state), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self.execute(root, data, state, False)
                self.assertFalse((root / "state.txt").exists())
                self.assertFalse((root / "report.txt").exists())


if __name__ == "__main__":
    unittest.main()
