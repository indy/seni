@echo off

setlocal EnableDelayedExpansion

rem builds a release build by default

if "%1" == "run" (
    cargo +nightly run
) else (
  if "%1" == "release" (
      cargo +nightly build --release
  ) else (
      cargo +nightly build
  )
)
