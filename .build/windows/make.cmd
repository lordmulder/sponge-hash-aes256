@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

REM --------------------------------------------------------------------------
REM Paths
REM --------------------------------------------------------------------------

if "%CARGO_INSTALL_PATH%" == "" (
	set "CARGO_INSTALL_PATH=%USERPROFILE%\.cargo\bin"
)

if "%SEVENZIP_INSTALL_PATH%" == "" (
	set "SEVENZIP_INSTALL_PATH=%ProgramFiles%\7-Zip"
)

if not exist "%CARGO_INSTALL_PATH%\cargo.exe" (
	echo File "%CARGO_INSTALL_PATH%\cargo.exe" not found. Please check CARGO_INSTALL_PATH and try again^^!
	goto:error
)

if not exist "%SEVENZIP_INSTALL_PATH%\7z.exe" (
	echo File "%SEVENZIP_INSTALL_PATH%\7z.exe" not found. Please check SEVENZIP_INSTALL_PATH and try again^^!
	goto:error
)

set "PATH=%CARGO_INSTALL_PATH%;%SEVENZIP_INSTALL_PATH%;%SystemRoot%\System32"

REM --------------------------------------------------------------------------
REM Clean-up
REM --------------------------------------------------------------------------

if exist "%CD%\target" (
	rmdir /S /Q "%CD%\target"
	if not !ERRORLEVEL! == 0 goto:error
)

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

for %%t in (x86_64 i686 aarch64) do (
	cargo build --release --target %%t-pc-windows-msvc
	if not !ERRORLEVEL! == 0 goto:error
)

cargo doc
if not %ERRORLEVEL% == 0 goto:error

popd

REM --------------------------------------------------------------------------
REM Packaging
REM --------------------------------------------------------------------------

mkdir "%CD%\target\dist"

for %%t in (x86_64 i686 aarch64) do (
	copy /B /Y "%CD%\..\..\app\target\%%t-pc-windows-msvc\release\sponge256sum.exe" "%CD%\target\dist\sponge256sum-%%t.exe"
	if not !ERRORLEVEL! == 0 goto:error
)

xcopy /E /H /I /Y "%CD%\..\..\app\target\doc" "%CD%\target\dist\doc"
if not %ERRORLEVEL% == 0 goto:error

copy /B /Y "%CD%\..\.resources\html\index.html" "%CD%\target\dist\doc\index.html"
if not %ERRORLEVEL% == 0 goto:error

pushd "%CD%\target\dist"
7z a -t7z -mx=9 "..\sponge256sum-%PKG_VERSION%-windows.7z" *
popd

attrib +R "%CD%\target\*.7z"

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
