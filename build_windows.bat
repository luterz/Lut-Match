@echo off
echo Compilazione LutMatch per Windows...
cargo build --release
copy target\release\lut-match.exe LutMatch.exe
echo Completato! Eseguibile generato: LutMatch.exe
pause
