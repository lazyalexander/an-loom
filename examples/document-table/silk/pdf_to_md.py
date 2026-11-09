"""Extract a text PDF into a Markdown document. OCR is outside this profile."""
import os
from pathlib import Path
from pypdf import PdfReader


def convert(source, target):
    reader = PdfReader(source)
    text = '\n\n'.join(page.extract_text() or '' for page in reader.pages)
    if not text.strip():
        raise ValueError('No extractable text; OCR requires another declared capability')
    target.write_text('# Extracted document\n\n' + text.rstrip() + '\n', encoding='utf-8')


if __name__ == '__main__':
    convert(Path(os.environ['AN_SPOOL_INPUT']), Path('document.md'))
