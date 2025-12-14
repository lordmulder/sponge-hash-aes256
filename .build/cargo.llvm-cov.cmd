@echo off
cd /d "%~dp0.."

for %%i in (lib app) do (
	echo -------------------------------
	echo -------------[%%i]-------------
	echo -------------------------------
	pushd "%%~i" || goto:error
	cargo clean || goto:error
	cargo llvm-cov --release --open -- --include-ignored || goto:error
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
