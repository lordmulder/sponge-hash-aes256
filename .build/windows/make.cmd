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

set "PATH=%SystemRoot%\System32;%SystemRoot%"
set "PATH=%CARGO_INSTALL_DIR%;%GIT_INSTALL_DIR%\cmd;%SEVENZIP_INSTALL_DIR%;%PATH%"

REM --------------------------------------------------------------------------
REM Clean-up
REM --------------------------------------------------------------------------

if exist "%CD%\target" (
	rmdir /S /Q "%CD%\target"
	if not !ERRORLEVEL! == 0 goto:error
)

mkdir "%CD%\target\dist"
if not %ERRORLEVEL% == 0 goto:error

set "DIST_DIR=%CD%\target\dist"
pushd "%CD%\..\..\app"

cargo clean
if not %ERRORLEVEL% == 0 goto:error

REM --------------------------------------------------------------------------
REM Detect version
REM --------------------------------------------------------------------------

set PKG_VERSION=

for /F "usebackq tokens=1,* delims=@" %%a in (`cargo pkgid`) do (
	set "PKG_VERSION=%%~b"
)

if "%PKG_VERSION%" == "" goto:error

REM --------------------------------------------------------------------------
REM Build
REM --------------------------------------------------------------------------

set "RUSTFLAGS=-Dwarnings -Ctarget-feature=+crt-static"

for %%t in (x86_64 i686 aarch64) do (
	cargo build --release --target %%t-pc-windows-msvc --features aligned --verbose
	if not !ERRORLEVEL! == 0 goto:error
	copy /B /Y "target\%%t-pc-windows-msvc\release\sponge256sum.exe" "%DIST_DIR%\sponge256sum-%%t.exe"
	if not !ERRORLEVEL! == 0 goto:error
)

for %%v in (v2 v3 v4) do (
	set "RUSTFLAGS=-Dwarnings -Ctarget-feature=+crt-static -Ctarget-cpu=x86-64-%%v"
	cargo build --release --target x86_64-pc-windows-msvc --features aligned --verbose
	if not !ERRORLEVEL! == 0 goto:error
	copy /B /Y "target\x86_64-pc-windows-msvc\release\sponge256sum.exe" "%DIST_DIR%\sponge256sum-x86_64-%%v.exe"
	if not !ERRORLEVEL! == 0 goto:error
)

cargo doc
if not %ERRORLEVEL% == 0 goto:error

cargo --version --verbose > "%CD%\target\.RUSTC_VERSION"
>> "%CD%\target\.RUSTC_VERSION" echo.
cargo rustc -- --version --verbose >> "%CD%\target\.RUSTC_VERSION"

popd

REM --------------------------------------------------------------------------
REM Create info
REM --------------------------------------------------------------------------

for /F "usebackq tokens=*" %%i in (`git describe --long --always --dirty`) do (
	> "%CD%\target\dist\BUILD_INFO.txt" echo Revision: %%i
)

>> "%CD%\target\dist\BUILD_INFO.txt" echo Built: %DATE% %TIME%
>> "%CD%\target\dist\BUILD_INFO.txt" echo.

type "%CD%\..\..\app\target\.RUSTC_VERSION" >> "%CD%\target\dist\BUILD_INFO.txt"
if not %ERRORLEVEL% == 0 goto:error

REM --------------------------------------------------------------------------
REM Packaging
REM --------------------------------------------------------------------------

copy /B /Y "%CD%\..\..\LICENSE" "%CD%\target\dist\LICENSE.txt"
if not %ERRORLEVEL% == 0 goto:error

xcopy /E /H /I /Y "%CD%\..\..\app\target\doc" "%CD%\target\dist\doc"
if not %ERRORLEVEL% == 0 goto:error

copy /B /Y "%CD%\..\..\.assets\html\index.html" "%CD%\target\dist\doc\index.html"
if not %ERRORLEVEL% == 0 goto:error

attrib +R "%CD%\target\*.*" /S
if not %ERRORLEVEL% == 0 goto:error

pushd "%CD%\target\dist"
7z a -t7z -mx=9 "..\sponge256sum-%PKG_VERSION%-windows.7z" *
popd

attrib +R "%CD%\target\*.7z"
if not %ERRORLEVEL% == 0 goto:error

copy /B /Y "%SEVENZIP_INSTALL_DIR%\7z.sfx" + "%CD%\target\sponge256sum-%PKG_VERSION%-windows.7z" "%CD%\target\sponge256sum-%PKG_VERSION%-windows.exe"
if not %ERRORLEVEL% == 0 goto:error

attrib +R "%CD%\target\*.exe"
if not %ERRORLEVEL% == 0 goto:error

REM --------------------------------------------------------------------------
REM Completed
REM --------------------------------------------------------------------------

echo Completed.
goto:eof

REM --------------------------------------------------------------------------
REM Error handler
REM --------------------------------------------------------------------------

:error
exit /B 1
