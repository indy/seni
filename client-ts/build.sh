#!/bin/sh

if [ "$1" = "sketch" ]
then
    tsc --project tsconfig-sketch.json
else
    tsc --project tsconfig-main.json
fi
