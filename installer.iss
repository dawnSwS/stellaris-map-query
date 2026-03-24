[Setup]
AppName=Stellaris Map Query
AppVersion=0.1.0
AppPublisher=Dawn
DefaultDirName={autopf}\Stellaris Map Query
DefaultGroupName=Stellaris Map Query
OutputDir=Output
OutputBaseFilename=StellarisMapQuery_Setup
Compression=lzma
SolidCompression=yes
ChangesAssociations=yes
DirExistsWarning=no
DisableProgramGroupPage=yes
DisableReadyPage=yes

[Files]
Source: "target\release\stellaris-map-query.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCR; Subkey: ".sav"; ValueType: string; ValueName: ""; ValueData: "StellarisSavFile"; Flags: uninsdeletevalue
Root: HKCR; Subkey: "StellarisSavFile"; ValueType: string; ValueName: ""; ValueData: "Stellaris 存档文件"; Flags: uninsdeletekey
Root: HKCR; Subkey: "StellarisSavFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\stellaris-map-query.exe,0"
Root: HKCR; Subkey: "StellarisSavFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\stellaris-map-query.exe"" ""%1"""