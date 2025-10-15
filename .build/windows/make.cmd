@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

REM --------------------------------------------------------------------------
REM Initialize paths
REM --------------------------------------------------------------------------

if "%CARGO_INSTALL_DIR%" == "" (
	set "CARGO_INSTALL_DIR=%USERPROFILE%\.cargo\bin"
)

if "%GIT_INSTALL_DIR%" == "" (
	set "GIT_INSTALL_DIR=%ProgramFiles%\Git"
)

if "%SEVENZIP_INSTALL_DIR%" == "" (
	set "SEVENZIP_INSTALL_DIR=%ProgramFiles%\7-Zip"
)

if "%NSIS_INSTALL_DIR%" == "" (
	set "NSIS_INSTALL_DIR=%ProgramFiles(x86)%\NSIS"
)

REM --------------------------------------------------------------------------
REM Check paths
REM --------------------------------------------------------------------------

if not exist "%CARGO_INSTALL_DIR%\cargo.exe" (
	echo File "%CARGO_INSTALL_DIR%\cargo.exe" not found. Please check CARGO_INSTALL_DIR and try again^^!
	goto:error
)

if not exist "%GIT_INSTALL_DIR%\cmd\git.exe" (
	echo File "%GIT_INSTALL_DIR%\cmd\git.exe" not found. Please check GIT_INSTALL_DIR and try again^^!
	goto:error
)

if not exist "%SEVENZIP_INSTALL_DIR%\7z.exe" (
	echo File "%SEVENZIP_INSTALL_DIR%\7z.exe" not found. Please check SEVENZIP_INSTALL_DIR and try again^^!
	goto:error
)

if not exist "%NSIS_INSTALL_DIR%\makensis.exe" (
	echo File "%NSIS_INSTALL_DIR%\makensis.exe" not found. Please check NSIS_INSTALL_DIR and try again^^!
	goto:error
)

set "PATH=%CD%\bin;%CARGO_INSTALL_DIR%;%GIT_INSTALL_DIR%\cmd;%SEVENZIP_INSTALL_DIR%;%NSIS_INSTALL_DIR%;%SystemRoot%\System32;%SystemRoot%"

REM --------------------------------------------------------------------------
REM Clean-up
REM --------------------------------------------------------------------------

if exist "%CD%\target" (
	rmdir /S /Q "%CD%\target"
	if exist "%CD%\target" (
		echo Failed to remove the existing "target" directory^^!
	)
)

mkdir "%CD%\target" || goto:error
mkdir "%CD%\target\dist" || goto:error

set "DIST_DIR=%CD%\target\dist"
pushd "%CD%\..\..\app"

REM --------------------------------------------------------------------------
REM Detect version
REM --------------------------------------------------------------------------

set PKG_VERSION=
set PKG_REGUUID=

for /F "usebackq tokens=1,* delims=@" %%a in (`cargo pkgid`) do (
	set "PKG_VERSION=%%~b"
)

for /f "usebackq" %%i in (`make_guid.exe`) do (
	set "PKG_REGUUID=%%~i"
)

if "%PKG_VERSION%" == "" goto:error
if "%PKG_REGUUID%" == "" goto:error

REM --------------------------------------------------------------------------
REM Build
REM --------------------------------------------------------------------------

set "DEFAULT_RUSTFLAGS=-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=..\.build\windows\res\app-icon.res"
set "RUSTFLAGS=%DEFAULT_RUSTFLAGS%"

for %%t in (x86_64 i686 aarch64) do (
	cargo clean || goto:error
	cargo build --release --target %%t-pc-windows-msvc --verbose || goto:error
	if "%%t" == "i686" (
		copy /B /Y "target\%%t-pc-windows-msvc\release\sponge256sum.exe" "%DIST_DIR%\sponge256sum-pentium4.exe" || goto:error
	) else (
		copy /B /Y "target\%%t-pc-windows-msvc\release\sponge256sum.exe" "%DIST_DIR%\sponge256sum-%%t.exe" || goto:error
	)
)

for %%v in (v2 v3 v4) do (
	set "RUSTFLAGS=%DEFAULT_RUSTFLAGS% -Ctarget-cpu=x86-64-%%v"
	cargo clean || goto:error
	cargo build --release --target x86_64-pc-windows-msvc --verbose || goto:error
	copy /B /Y "target\x86_64-pc-windows-msvc\release\sponge256sum.exe" "%DIST_DIR%\sponge256sum-x86_64-%%v.exe" || goto:error
)

set "RUSTFLAGS=-Dwarnings"
cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256 || goto:error

cargo --version --verbose > "%CD%\target\.RUSTC_VERSION"
>> "%CD%\target\.RUSTC_VERSION" echo.
cargo rustc -- --version --verbose >> "%CD%\target\.RUSTC_VERSION"

popd

REM --------------------------------------------------------------------------
REM Create info
REM --------------------------------------------------------------------------

for /F "usebackq tokens=*" %%i in (`git describe --long --tags --always --dirty`) do (
	> "%CD%\target\dist\BUILD_INFO.txt" echo Revision: %%i
)

>> "%CD%\target\dist\BUILD_INFO.txt" echo Built: %DATE% %TIME%
>> "%CD%\target\dist\BUILD_INFO.txt" echo.

type "%CD%\..\..\app\target\.RUSTC_VERSION" >> "%CD%\target\dist\BUILD_INFO.txt"

REM --------------------------------------------------------------------------
REM Packaging
REM --------------------------------------------------------------------------

xcopy /E /H /I /Y "%CD%\..\..\app\target\doc" "%CD%\target\dist\doc" || goto:error
copy /B /Y "%CD%\..\..\.assets\html\index.html" "%CD%\target\dist\doc\index.html" || goto:error
copy /B /Y "%CD%\..\..\LICENSE" "%CD%\target\dist\LICENSE.txt" || goto:error

attrib +R "%CD%\target\*.*" /S || goto:error

pushd "%CD%\target\dist"
7z a -t7z -mx=9 "..\sponge256sum-%PKG_VERSION%-windows.7z" * || goto:error
popd

attrib +R "%CD%\target\*.7z"  || goto:error

makensis "-DOUTPUT_FILE=%CD%\target\sponge256sum-%PKG_VERSION%-windows.exe" "-DSOURCE_PATH=%CD%\target\dist" "-DPKG_VERSION=%PKG_VERSION%" "-DPKG_REGUUID=%PKG_REGUUID%" "%CD%\installer\installer.nsi" || goto:error

attrib +R "%CD%\target\*.exe" || goto:error

REM --------------------------------------------------------------------------
REM Completed
REM --------------------------------------------------------------------------

echo Completed.
goto:eof

REM --------------------------------------------------------------------------
REM Error handler
REM --------------------------------------------------------------------------

:error
echo Error: Something went wrong^^!
exit /B 1
