import subprocess
import tempfile
from pathlib import Path
import unittest

SCRIPT = Path(__file__).resolve().parents[1]/'transform.sh'


class Summary(unittest.TestCase):
    def test_counts_and_empty_input(self):
        for text, expected in [('', 'total=0\nINFO=0\nWARN=0\nERROR=0\n'),
                               ('INFO x\nERROR y\nINFO z\n', 'total=3\nINFO=2\nWARN=0\nERROR=1\n')]:
            with tempfile.NamedTemporaryFile(mode='w') as source:
                source.write(text)
                source.flush()
                result = subprocess.run(['sh', str(SCRIPT), source.name], capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 0)
                self.assertEqual(result.stdout, expected)

    def test_unknown_level_refuses(self):
        with tempfile.NamedTemporaryFile(mode='w') as source:
            source.write('UNKNOWN event\n')
            source.flush()
            result = subprocess.run(['sh', str(SCRIPT), source.name], capture_output=True, timeout=5)
            self.assertNotEqual(result.returncode, 0)
