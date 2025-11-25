@echo off
cd /d "%~dp0.."

for %%i in (lib app) do (
	echo -------------------------------
	echo -------------[%%i]-------------
	echo -------------------------------
	pushd "%%~i" || goto:error
	cargo clean || goto:error
	cargo upgrade || goto:error
	cargo update || goto:error
	cargo fmt || goto:error
	cargo clippy || goto:error
	cargo build --release || goto:error
	popd || goto:error
)

echo.
echo Completed successfully.
echo.
pause
goto:eof

:error
echo.
echo Error: Something went wrong!
echo.
pause
exit /b 1
