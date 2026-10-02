"""Check distribution contents without launching the game."""
import argparse, hashlib, re, zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser()
p.add_argument('--analog-yaw-zip',type=Path,default=root.parent/'AC8AnalogYaw/dist/AC8AnalogYaw-0.1.3.zip')
args=p.parse_args()
with zipfile.ZipFile(args.analog_yaw_zip) as upstream:
    for name in ('AC8HOTAS-0.1.1.zip','AC8HOTAS-0.1.1-with-AC8AnalogYaw-0.1.3.zip'):
        with zipfile.ZipFile(root/'dist'/name) as z:
            assert z.testzip() is None
            names=z.namelist()
            assert len(names)==len(set(names))
            assert 'INSTALL.md' in names and 'AC8HOTAS/Scripts/actions.lua' in names
            assert not any(n.endswith(('mods.txt','mods.json','devices.lua','.log','.bak','throttle_trace.lua')) for n in names)
            config=z.read('AC8HOTAS/bindings.lua').decode('utf-8')
            actions=re.findall(r'^    (\w+) = \d+,',z.read('AC8HOTAS/Scripts/actions.lua').decode(),re.M)
            nils=re.findall(r'^\s*(\w+)\s*=\s*nil,',config,re.M)
            assert len(actions)==49 and sorted(nils)==sorted(actions), 'Default must list every action once, unbound'
            assert not re.search(r'\{[0-9A-Fa-f]{8}-[0-9A-Fa-f-]{27,}\}',config), 'Personal device GUID leaked'
            assert 'BEGIN DETECTED DEVICES' not in config
            assert 'CC0 1.0 Universal' in z.read('AC8HOTAS/LICENSE').decode()
            yaw=[n for n in names if n.startswith('AC8AnalogYaw/')]
            if 'with-' in name:
                assert len(yaw)==6
                for n in yaw: assert z.read(n)==upstream.read(n), 'Bundled dependency changed: '+n
            else: assert not yaw
with zipfile.ZipFile(root/'dist/AC8HOTAS-0.1.1-source.zip') as z:
    assert z.testzip() is None
    assert not any(n.endswith(('.dll','.exe','.log','.bak')) for n in z.namelist())
for line in (root/'dist/SHA256SUMS.txt').read_text().splitlines():
    digest,name=line.split('  ',1)
    assert hashlib.sha256((root/'dist'/name).read_bytes()).hexdigest().upper()==digest
print('PASS: 49 unbound actions, no personal GUIDs/runtime files, licenses, unchanged yaw bundle, source archive and checksums.')
