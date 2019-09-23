@echo off

setlocal EnableDelayedExpansion

rem builds a release build by default

if "%1" == "release" (
    cargo run --release
) else (
    cargo run
)
