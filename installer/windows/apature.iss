; Apature Windows installer (Inno Setup 6). Built in CI by the release
; workflow; ISCC.exe is preinstalled on GitHub's windows runners.
;
;   iscc /DAppVersion=0.1.1 /DSourceDir=..\..\target\release installer\windows\apature.iss
;
; Installs to Program Files, adds a Start menu entry and an optional desktop
; icon, registers an uninstaller. Per-user settings stay in %APPDATA% and are
; never removed by the uninstaller.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\target\release"
#endif

[Setup]
AppId={{8A5D5C3E-5B0F-4C6B-9B1E-2F4D7A9C1E03}
AppName=Apature
AppVersion={#AppVersion}
AppVerName=Apature {#AppVersion}
AppPublisher=Caleb McCartney
AppPublisherURL=https://github.com/McCal-Codes/apature
AppSupportURL=https://github.com/McCal-Codes/apature/issues
AppUpdatesURL=https://github.com/McCal-Codes/apature/releases
DefaultDirName={autopf}\Apature
DefaultGroupName=Apature
UninstallDisplayIcon={app}\apature.exe
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\..\target\bundle
OutputBaseFilename=Apature-{#AppVersion}-windows-x86_64-setup
SetupIconFile=..\..\apps\desktop\assets\icon.ico
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
LicenseFile=..\..\LICENSE-MIT

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "{#SourceDir}\apature.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\apature-cli.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Apature"; Filename: "{app}\apature.exe"
Name: "{group}\Uninstall Apature"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Apature"; Filename: "{app}\apature.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\apature.exe"; Description: "Launch Apature"; Flags: nowait postinstall skipifsilent
