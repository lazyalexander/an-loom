"""Package two domain programs and select an operator-owned Python runtime."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(loom, host, python, root):
    pins = dict(line.split('==') for line in (HERE / 'requirements.txt').read_text().splitlines())
    code = "import sys,site,json,importlib.metadata as m; print(json.dumps({'exe':sys.executable,'base':sys.base_prefix,'site':site.getsitepackages()[0],'versions':{n:m.version(n) for n in sys.argv[1:]}}))"
    info = json.loads(subprocess.check_output([str(python), '-B', '-s', '-c', code, *pins], text=True))
    assert info['versions'] == pins, 'Use the pinned document environment'
    site = Path(info['site']).resolve()
    files = {}
    for path in sorted(site.rglob('*')):
        assert not path.is_symlink(), 'Runtime dependencies must be regular files'
        if path.is_file():
            files[path.relative_to(site).as_posix()] = digest(path)
    deps = sorted(f'{k}-{v}' for k,v in pins.items())
    runtime = root / 'runtime.json'
    runtime.write_text(json.dumps({'schema':'an-native-python-runtime-v1','executable':str(Path(info['exe']).resolve()),
        'executable_sha256':digest(Path(info['exe']).resolve()),'stdlib':str(Path(info['base']).resolve()),
        'site':str(site),'site_sha256':hashlib.sha256(json.dumps(files,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'dependencies':deps}))
    instances = []
    for stage, output in [('pdf_to_md','document.md'), ('md_to_excel','result.xlsx')]:
        project = root / (stage + '-project')
        (project / 'silk').mkdir(parents=True)
        shutil.copyfile(HERE / 'silk' / (stage + '.py'), project / 'silk/main.py')
        if stage == 'md_to_excel':
            shutil.copyfile(HERE / 'silk/ooxml.py', project / 'silk/ooxml.py')
        code_files = ['silk/main.py'] + (['silk/ooxml.py'] if stage == 'md_to_excel' else [])
        (project / 'loom.toml').write_text(f'''schema_version = 1
name = "{stage.replace('_', '-')}"
version = "0.1.0"
description = "Reference document conversion"
include = ["loom.toml", "native-spool.toml", "silk"]
''')
        binary_outputs = [output] if output.endswith('xlsx') else []
        (project / 'native-spool.toml').write_text(f'''schema = "an-native-spool-v1"
runtime = "macos-native-python-v1"
entry = "silk/main.py"
silk = {json.dumps(code_files)}
kit = []
editable = []
inputs = ["input"]
outputs = {json.dumps([output])}
binary_outputs = {json.dumps(binary_outputs)}
capabilities = ["read-input", "write-declared-outputs"]
dependencies = {json.dumps(deps)}
acceptance = ["python3 -B silk/main.py"]

[events]
consumes = []
produces = ["artifact.accepted"]
''')
        def run(binary,*args):
            p = subprocess.run([str(binary),*map(str,args)],capture_output=True,text=True,timeout=60)
            assert p.returncode == 0, p.stderr
            return json.loads(p.stdout)
        run(loom,'check',project)
        archive = run(loom,'pack',project)['archive']
        run(loom,'verify',archive)
        release, instance = root / (stage+'-release'), root / stage
        run(loom,'export-native-spool-v3',archive,release)
        run(host,'spool-load',release,instance,'--runtime',runtime)
        instances.append(instance)
    return instances
