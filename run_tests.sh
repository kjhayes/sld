#! /usr/bin/bash

SLD=./target/debug/sld

gcc -c ./tests/main1.c -o ./tests/main1.o
gcc -c ./tests/obj1.c -o ./tests/obj1.o

# Add .exe even on linux for .gitignore
$SLD ./tests/obj1.o ./tests/main1.o -o ./tests/test1.exe

