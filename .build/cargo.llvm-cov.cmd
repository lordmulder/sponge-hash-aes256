@echo off
cd /d "%~dp0.."

cargo clean
cargo llvm-cov --features with-mimalloc --workspace --open -- --include-ignored

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
