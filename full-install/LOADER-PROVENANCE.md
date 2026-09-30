# Included loader

UE4SS reports v3.0.1 Beta, commit 03dbd5c0 in the tested development installation.
The UE4SS.dll and dwmapi.dll files are copied byte for byte from that working
installation, originally supplied with the user's FOV mod. They have not been
independently matched to an official release archive; this package does not claim
to be an official UE4SS release. Packaging pins their SHA-256 fingerprints:

| File | SHA-256 |
| --- | --- |
| dwmapi.dll | C5D2AB9F9B89BD94460B0A283EEFB113085105014011CAC961F36787376DB744 |
| UE4SS.dll | 3FC1CD877007499E8C4F2152E12F76098ABA8E267B7EE65846EDC17CF7A0D1A4 |

Upstream source: https://github.com/UE4SS-RE/RE-UE4SS

UE4SS uses the MIT license, copyright 2022 Narknon. The license supplied with
the working installation is retained as Game/Binaries/Win64/UE4SS/LICENSE.
Upstream license: https://github.com/UE4SS-RE/RE-UE4SS/blob/main/LICENSE

The config is based on the working loader settings with developer hot reload
disabled, no extra mod paths, no automatic Lua reload and no debug consoles.
The package selects UE4SS using override.txt and includes a new mod list enabling
only AC8HOTAS and AC8AnalogYaw. No original FOV scripts or configuration are included.
