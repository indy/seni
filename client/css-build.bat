@echo off

setlocal EnableDelayedExpansion

cargo run --manifest-path scss\Cargo.toml -- scss\scss\seni.scss ..\www\stylesheet.css
