# SPDX-License-Identifier: 0BSD
# SpongeHash-AES256
# Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

Unicode true
RequestExecutionLevel user

!ifndef OUTPUT_FILE
  !error "Error: OUTPUT_FILE is not defined!"
!endif

!ifndef SOURCE_DIR
  !error "Error: SOURCE_DIR is not defined!"
!endif

!ifndef PKG_VERSION
  !error "Error: PKG_VERSION is not defined!"
!endif

Name "sponge256sum ${PKG_VERSION}"
OutFile "${OUTPUT_FILE}"
InstallDir "$DESKTOP\sponge256sum-${PKG_VERSION}"

SetCompressor /SOLID lzma
SetCompressorDictSize 64

Icon "icon.ico"
LicenseData "license.rtf"

VIProductVersion "${PKG_VERSION}.0"
VIFileVersion "${PKG_VERSION}.0"
VIAddVersionKey /LANG=1033 "CompanyName" "LoRd_MuldeR <mulder2@gmx.de>"
VIAddVersionKey /LANG=1033 "FileDescription" "sponge256hash"
VIAddVersionKey /LANG=1033 "FileVersion" "${PKG_VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>"
VIAddVersionKey /LANG=1033 "ProductName" "SpongeHash-AES256"
VIAddVersionKey /LANG=1033 "ProductVersion" "${PKG_VERSION}"

Page license
Page directory
Page instfiles

Section ""
  SetOutPath "$INSTDIR"
  File /r "${SOURCE_DIR}\*.*"
SectionEnd

Function .onInstSuccess
  ExecShell "open" "$INSTDIR"
FunctionEnd
