@echo off

setlocal EnableDelayedExpansion

rem builds index.js by default

if "%1" == "sketch" (
    tsc --project tsconfig-sketch.json
) else (
    tsc --project tsconfig-main.json
)
