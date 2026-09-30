"""Verify the fresh-install layout and exact dependency contents offline."""
import hashlib
import json
import zipfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
prefix = 'Game/Binaries/Win64/'
with zipfile.ZipFile(root/'dist/AC8HOTAS-0.1.0-full-install.zip') as z, zipfile.ZipFile(root/'dist/AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip') as mods:
    assert z.testzip() is None
    names = z.namelist()
    assert len(names) == len(set(names))
    expected = {'INSTALL-FULL.md', 'LOADER-PROVENANCE.md'}
    expected.update(prefix+n for n in ('dwmapi.dll', 'override.txt', 'UE4SS/UE4SS.dll', 'UE4SS/LICENSE', 'UE4SS/UE4SS-settings.ini', 'UE4SS/Mods/mods.txt', 'UE4SS/Mods/mods.json'))
    for name in mods.namelist():
        if name.startswith(('AC8HOTAS/', 'AC8AnalogYaw/')):
            target = prefix+'UE4SS/Mods/'+name
            expected.add(target)
            assert z.read(target) == mods.read(name), name
    assert set(names) == expected, 'Unexpected or missing file'
    pins = {'dwmapi.dll': 'C5D2AB9F9B89BD94460B0A283EEFB113085105014011CAC961F36787376DB744', 'UE4SS/UE4SS.dll': '3FC1CD877007499E8C4F2152E12F76098ABA8E267B7EE65846EDC17CF7A0D1A4'}
    for name, digest in pins.items():
        assert hashlib.sha256(z.read(prefix+name)).hexdigest().upper() == digest
    assert z.read(prefix+'override.txt').decode().strip() == 'UE4SS'
    assert z.read(prefix+'UE4SS/Mods/mods.txt').decode().splitlines() == ['AC8HOTAS : 1', 'AC8AnalogYaw : 1']
    assert json.loads(z.read(prefix+'UE4SS/Mods/mods.json')) == [{'mod_name':'AC8HOTAS','mod_enabled':True},{'mod_name':'AC8AnalogYaw','mod_enabled':True}]
    settings = z.read(prefix+'UE4SS/UE4SS-settings.ini').decode()
    for option in ('EnableHotReloadSystem = 0', 'EnableAutoReloadingLuaMods = 0', 'ConsoleEnabled = 0', 'GuiConsoleEnabled = 0', 'bUseUObjectArrayCache = false'):
        assert option in settings
    assert 'MIT License' in z.read(prefix+'UE4SS/LICENSE').decode()
    assert not any(n.endswith(('.log','.bak','.exe','devices.lua')) for n in names)
for line in (root/'dist/SHA256SUMS.txt').read_text().splitlines():
    digest, name = line.split('  ', 1)
    assert hashlib.sha256((root/'dist'/name).read_bytes()).hexdigest().upper() == digest
print('PASS: full-install layout, exact mod contents, pinned loader binaries, clean settings, enabled mods, licenses and checksums.')
