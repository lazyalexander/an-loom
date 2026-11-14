"""Create a typed workbook from the reference domain's explicit Markdown table."""
from decimal import Decimal
import os
from pathlib import Path
import ooxml  # Initialize the existing deterministic document MIME adapter.
from openpyxl import Workbook


def rows(source):
    lines = [line.strip() for line in source.read_text(encoding='utf-8').splitlines() if line.strip().startswith('|')]
    if len(lines) < 3 or [v.strip() for v in lines[0].strip('|').split('|')] != ['Item', 'Quantity', 'Unit price']:
        raise ValueError('Expected the declared Item, Quantity, Unit price table')
    result = []
    for line in lines[2:]:
        name, quantity, price = [v.strip() for v in line.strip('|').split('|')]
        quantity, price = int(quantity), Decimal(price)
        if not name or quantity < 0 or not price.is_finite() or price < 0:
            raise ValueError('Invalid table row')
        result.append((name, quantity, price))
    return result


def convert(source, target):
    values = rows(source)
    workbook = Workbook()
    sheet = workbook.active
    sheet.title = 'Parts'
    sheet.append(['Item', 'Quantity', 'Unit price', 'Amount'])
    for name, quantity, price in values:
        sheet.append([name, quantity, float(price), float(quantity * price)])
    sheet.append(['Total', None, None, float(sum(q*p for _, q, p in values))])
    sheet.column_dimensions['A'].width = 24
    for row in sheet.iter_rows(min_row=2, min_col=3, max_col=4):
        for cell in row:
            cell.number_format = '0.00'
    workbook.save(target)


if __name__ == '__main__':
    convert(Path(os.environ['AN_SPOOL_INPUT']), Path('result.xlsx'))
