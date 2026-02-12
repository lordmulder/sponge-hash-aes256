# SPDX-License-Identifier: 0BSD
# SpongeHash-AES256
# Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

!include 'LogicLib.nsh'

!ifndef OUTPUT_FILE
  !error "Error: OUTPUT_FILE is not defined!"
!endif

!ifndef SOURCE_PATH
  !error "Error: SOURCE_PATH is not defined!"
!endif

!ifndef PKG_VERSION
  !error "Error: PKG_VERSION is not defined!"
!endif

!ifndef PKG_REGUUID
  !error "Error: PKG_REGUUID is not defined!"
!endif

!define /utcdate BUILD_TIME "%H:%M:%S"
!define /utcdate BUILD_DATE "%Y-%m-%d"
!define REG_INSTPATH `"Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_REGUUID}"`

Unicode true
RequestExecutionLevel highest
XPStyle on
ManifestSupportedOS all

Name "sponge256sum ${PKG_VERSION}"
BrandingText "Built on ${BUILD_DATE} at ${BUILD_TIME}"

OutFile "${OUTPUT_FILE}"

InstallDirRegKey HKCU ${REG_INSTPATH} "InstallLocation"
InstallDir "$DESKTOP\sponge256sum-${PKG_VERSION}"

SetCompressor /SOLID lzma
SetCompressorDictSize 64

Icon "resources\app-icon.ico"
LicenseData "resources\LICENSE.rtf"
LicenseForceSelection checkbox

VIProductVersion "${PKG_VERSION}.0"
VIFileVersion "${PKG_VERSION}.0"
VIAddVersionKey /LANG=1033 "CompanyName" "LoRd_MuldeR <mulder2@gmx.de>"
VIAddVersionKey /LANG=1033 "FileDescription" "sponge256hash"
VIAddVersionKey /LANG=1033 "FileVersion" "${PKG_VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>"
VIAddVersionKey /LANG=1033 "ProductName" "SpongeHash-AES256"
VIAddVersionKey /LANG=1033 "ProductVersion" "${PKG_VERSION}"

Page license
Page directory
Page components
Page instfiles

!macro _SetOutPath DIR
  ClearErrors
  SetOutPath `${DIR}`
  ${If} ${Errors}
    MessageBox MB_ICONSTOP|MB_TOPMOST "Failed to create directory:$\n$\n${DIR}" /SD IDOK
    Abort "Failed to create directory: ${DIR}"
  ${EndIf}
!macroend

Section "Program files (required)"
  SectionIn RO
  !insertmacro _SetOutPath "$INSTDIR"
  File "${SOURCE_PATH}\*.*"
SectionEnd

Section "Program files for old Windows versions"
  !insertmacro _SetOutPath "$INSTDIR\extra"
  File "${SOURCE_PATH}\extra\*.*"
SectionEnd

Section "Documentation"
  !insertmacro _SetOutPath "$INSTDIR\doc"
  File /r "${SOURCE_PATH}\doc\*.*"
SectionEnd

Section ""
  WriteRegDWORD HKCU ${REG_INSTPATH} "Installed" 1
  WriteRegStr HKCU ${REG_INSTPATH} "InstallLocation" "$INSTDIR"
SectionEnd

Function .onInstSuccess
  ${IfNot} ${Silent}
    ExecShell "open" "$INSTDIR"
  ${EndIf}
FunctionEnd
