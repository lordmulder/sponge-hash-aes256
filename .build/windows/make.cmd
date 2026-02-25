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

if "%XORRISO_INSTALL_DIR%" == "" (
	set "XORRISO_INSTALL_DIR=C:\msys64\usr\bin"
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

if not exist "%NSIS_INSTALL_DIR%\makensis.exe" (
	echo File "%NSIS_INSTALL_DIR%\makensis.exe" not found. Please check NSIS_INSTALL_DIR and try again^^!
	goto:error
)

if not exist "%XORRISO_INSTALL_DIR%\xorriso.exe" (
	echo File "%XORRISO_INSTALL_DIR%\xorriso.exe" not found. Please check XORRISO_INSTALL_DIR and try again^^!
	goto:error
)

if exist "%SEVENZIP_INSTALL_DIR%\7za.exe" (
	set SEVENZIP=7za.exe
) else (
	if exist "%SEVENZIP_INSTALL_DIR%\7z.exe" (
		set SEVENZIP=7z.exe
	) else (
		echo File "%SEVENZIP_INSTALL_DIR%\7z[a].exe" not found. Please check SEVENZIP_INSTALL_DIR and try again^^!
		goto:error
	)
)

set "PATH=%CD%\bin;%CARGO_INSTALL_DIR%;%GIT_INSTALL_DIR%\cmd;%SEVENZIP_INSTALL_DIR%;%NSIS_INSTALL_DIR%;%XORRISO_INSTALL_DIR%;%SystemRoot%\System32;%SystemRoot%"

REM --------------------------------------------------------------------------
REM Check the Rust version
REM --------------------------------------------------------------------------

set VER_CARGO_MAJOR=0
set VER_CARGO_MINOR=0

for /F "usebackq tokens=1,2" %%a in (`cargo version`) do (
	if "%%~a" == "cargo" (
		for /F "tokens=1,2 delims=." %%i in ("%%~b") do (
			set "VER_CARGO_MAJOR=%%~i"
			set "VER_CARGO_MINOR=%%~j"
		)
	)
)

if %VER_CARGO_MAJOR% neq 1 (
	echo Rust toolchain version could not be detected or is too old^^!
	goto:error
)

if %VER_CARGO_MINOR% lss 89 (
	echo Rust toolchain version could not be detected or is too old^^!
	goto:error
)

REM --------------------------------------------------------------------------
REM Check for uncommitted changes
REM --------------------------------------------------------------------------

git describe --long --tags --always --dirty || goto:error

for /F "usebackq delims=" %%a in (`git status --porcelain`) do (
	echo Git: Uncommitted changes detected. Cowardly refusing to create an empty archive^^!
	goto:error
)

REM --------------------------------------------------------------------------
REM Clean-up
REM --------------------------------------------------------------------------

if exist "out\target" (
	rmdir /S /Q "out\target"
	if exist "out\target" (
		echo Failed to remove the existing "target" directory^^!
	)
)

mkdir "out\target" || goto:error
mkdir "out\target\release" || goto:error

:retry_mktemp
set "CARGO_TARGET_DIR=%TEMP%\tmp_%RANDOM%"
if exist "%CARGO_TARGET_DIR%" goto:retry_mktemp
mkdir "%CARGO_TARGET_DIR%" || goto:retry_mktemp

REM --------------------------------------------------------------------------
REM Detect version
REM --------------------------------------------------------------------------

set PKG_VERSION=
set PKG_REGUUID=

for /F "usebackq tokens=1,* delims=@" %%a in (`cargo pkgid --package sponge256sum`) do (
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

set RUSTC_BOOTSTRAP=

set "DEFAULT_RUSTFLAGS=-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Ccodegen-units=1 -Clto=fat -Cdebuginfo=none -Cpanic=abort -Clink-arg=/DEBUG:NONE"
set "RUSTFLAGS=%DEFAULT_RUSTFLAGS%"

for %%t in (x86_64 i686 aarch64) do (
	cargo clean || goto:error
	cargo build --release --target %%~t-pc-windows-msvc --features with-mimalloc --verbose || goto:error
	if "%%~t" == "i686" (
		copy /B /Y "%CARGO_TARGET_DIR%\%%t-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-i686-sse2.exe" || goto:error
	) else (
		copy /B /Y "%CARGO_TARGET_DIR%\%%t-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-%%~t.exe" || goto:error
	)
)

for %%t in (x86_64 i686) do (
	set "RUSTFLAGS=%DEFAULT_RUSTFLAGS% -Ctarget-feature=+aes"
	cargo clean || goto:error
	cargo build --release --target %%~t-pc-windows-msvc --features with-mimalloc --verbose || goto:error
	if "%%~t" == "i686" (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-i686-sse2-aes.exe" || goto:error
	) else (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-%%~t-aes.exe" || goto:error
	)
)

for %%t in ("" aes) do (
	if "%%~t" == "aes" (
		set "RUSTFLAGS=%DEFAULT_RUSTFLAGS% -Ctarget-cpu=x86-64-v3 -Ctarget-feature=+aes"
	) else (
		set "RUSTFLAGS=%DEFAULT_RUSTFLAGS% -Ctarget-cpu=x86-64-v3"
	)
	cargo clean || goto:error
	cargo build --release --target x86_64-pc-windows-msvc --features with-mimalloc --verbose || goto:error
	if "%%~t" == "aes" (
		copy /B /Y "%CARGO_TARGET_DIR%\x86_64-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-x86_64-v3-aes.exe" || goto:error
	) else (
		copy /B /Y "%CARGO_TARGET_DIR%\x86_64-pc-windows-msvc\release\sponge256sum.exe" "out\target\release\sponge256sum-x86_64-v3.exe" || goto:error
	)
)

REM ~~~~~~~~~~~~~~~~~~~~~~~~~~~~
REM Windows 7
REM ~~~~~~~~~~~~~~~~~~~~~~~~~~~~

mkdir "out\target\release\legacy-compat" || goto:error

set RUSTC_BOOTSTRAP=1
set "RUSTFLAGS=%DEFAULT_RUSTFLAGS%"

for %%t in (x86_64 i686) do (
	cargo clean || goto:error
	cargo build -Zbuild-std=std,panic_abort --release --target %%~t-win7-windows-msvc --verbose || goto:error
	if "%%~t" == "i686" (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-win7-windows-msvc\release\sponge256sum.exe" "out\target\release\legacy-compat\sponge256sum-win7-i686-sse2.exe" || goto:error
	) else (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-win7-windows-msvc\release\sponge256sum.exe" "out\target\release\legacy-compat\sponge256sum-win7-%%~t.exe" || goto:error
	)
)

for %%t in (x86_64 i686) do (
	set "RUSTFLAGS=%DEFAULT_RUSTFLAGS% -Ctarget-feature=+aes"
	cargo clean || goto:error
	cargo build -Zbuild-std=std,panic_abort --release --target %%~t-win7-windows-msvc --verbose || goto:error
	if "%%~t" == "i686" (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-win7-windows-msvc\release\sponge256sum.exe" "out\target\release\legacy-compat\sponge256sum-win7-i686-sse2-aes.exe" || goto:error
	) else (
		copy /B /Y "%CARGO_TARGET_DIR%\%%~t-win7-windows-msvc\release\sponge256sum.exe" "out\target\release\legacy-compat\sponge256sum-win7-%%~t-aes.exe" || goto:error
	)
)

REM ~~~~~~~~~~~~~~~~~~~~~~~~~~~~
REM Docs
REM ~~~~~~~~~~~~~~~~~~~~~~~~~~~~

set "RUSTFLAGS=-Dwarnings"

cargo clean || goto:error
cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256 || goto:error
xcopy /E /H /I /Y "%CARGO_TARGET_DIR%\doc" "out\target\release\doc" || goto:error

cargo --version --verbose > "%CARGO_TARGET_DIR%\.RUSTC_VERSION"
>> "%CARGO_TARGET_DIR%\.RUSTC_VERSION" echo.
cargo rustc --manifest-path "%CD%\..\.auxiliary-files\blank-project/Cargo.toml" -- --version --verbose >> "%CARGO_TARGET_DIR%\.RUSTC_VERSION"

REM --------------------------------------------------------------------------
REM Create info
REM --------------------------------------------------------------------------

for /F "usebackq tokens=*" %%i in (`git describe --long --tags --always --dirty`) do (
	> "out\target\release\BUILD_INFO.txt" echo Revision: %%i
)

>> "out\target\release\BUILD_INFO.txt" echo Built: %DATE% %TIME%
>> "out\target\release\BUILD_INFO.txt" echo.

type "%CARGO_TARGET_DIR%\.RUSTC_VERSION" >> "out\target\release\BUILD_INFO.txt"

REM --------------------------------------------------------------------------
REM Packaging
REM --------------------------------------------------------------------------

copy /B /Y "..\..\LICENSE" "out\target\release\LICENSE.txt" || goto:error
copy /B /Y "..\..\.assets\html\index.html" "out\target\release\doc\index.html" || goto:error
attrib +R "out\target\release\*.*" /S || goto:error

pushd "out\target\release"
%SEVENZIP% a -tzip -mx=9 -mfb=258 -mpass=15 "..\sponge256sum-%PKG_VERSION%-windows.zip" * || goto:error
popd
attrib +R "out\target\*.zip" || goto:error

makensis.exe -NOCD -WX -INPUTCHARSET UTF8 "-DOUTPUT_FILE=out\target\sponge256sum-%PKG_VERSION%-windows.exe" "-DSOURCE_PATH=out\target\release" "-DPKG_VERSION=%PKG_VERSION%" "-DPKG_REGUUID=%PKG_REGUUID%" "resources\build.nsi" || goto:error
attrib +R "out\target\*.exe" || goto:error

xorriso.exe -as mkisofs -iso-level 3 -R -J -V "sponge256sum" -o "out\target\sponge256sum-%PKG_VERSION%-windows.iso" "out/target/release" "resources/autorun.inf" || goto:error
attrib +R "out\target\*.iso" || goto:error

REM --------------------------------------------------------------------------
REM Completed
REM --------------------------------------------------------------------------

rmdir /S /Q "%CARGO_TARGET_DIR%"

echo Completed.
goto:eof

REM --------------------------------------------------------------------------
REM Error handler
REM --------------------------------------------------------------------------

:error
echo Error: Something went wrong^^!
exit /B 1
