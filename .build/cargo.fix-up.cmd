@echo off
cd /d "%~dp0.."

cargo clean || goto:error
cargo upgrade || goto:error
cargo update --workspace || goto:error
cargo fmt --all || goto:error
cargo clippy --workspace || goto:error
cargo build --workspace --release || goto:error

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
