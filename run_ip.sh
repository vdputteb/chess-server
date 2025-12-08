@echo off
echo Ton IP locale :
ipconfig | findstr /R /C:"IPv4"
pause