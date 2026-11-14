import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
import sys
from openpyxl import load_workbook

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE / 'silk'))
spec = importlib.util.spec_from_file_location('table', HERE / 'silk/md_to_excel.py')
table = importlib.util.module_from_spec(spec)
spec.loader.exec_module(table)


class Conversion(unittest.TestCase):
    def test_typed_rows_and_total(self):
        with TemporaryDirectory() as d:
            source, output = Path(d)/'in.md', Path(d)/'out.xlsx'
            source.write_text('| Item | Quantity | Unit price |\n| --- | --- | --- |\n| Bolt | 12 | 0.5 |\n| Plate | 3 | 8 |\n| Bearing | 2 | 4.25 |\n')
            table.convert(source, output)
            rows = list(load_workbook(output, data_only=True).active.values)
            self.assertEqual(rows[1], ('Bolt',12,.5,6))
            self.assertEqual(rows[-1][-1],38.5)
            source.write_text('| Item | Quantity | Unit price |\n| --- | --- | --- |\n| Bolt | 2 | NaN |\n')
            with self.assertRaises(ValueError): table.rows(source)
