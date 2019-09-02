@echo off

setlocal EnableDelayedExpansion

rem builds a release build by default

if "%1" == "run" (
    cargo +nightly run
) else (
    cargo +nightly build
)
