@echo off
setlocal enabledelayedexpansion

set "file=assets.wxs"
set "tempfile=tmp.wxs"

:: Find the platform directory (e.g., x86_64-pc-windows-msvc).
for /d %%i in (..\..\..\target\dx\unavi-launcher\release\*) do (
    set "platform_dir=%%~nxi"
)

set "assets_path=..\..\..\target\dx\unavi-launcher\release\!platform_dir!\app\assets"

echo Platform directory: !platform_dir!
echo Assets path: !assets_path!

heat dir "!assets_path!" -cg AssetsGroup -dr AssetsFolder -ag -sfrag -out %file%

if errorlevel 1 (
    echo ERROR: heat command failed
    exit /b 1
)

:: Replace SourceDir with workspace-relative path using PowerShell.
powershell -Command "(Get-Content '%file%') -replace 'SourceDir', 'target\dx\unavi-launcher\release\!platform_dir!\app\assets' | Set-Content '%tempfile%'"

move /y "%tempfile%" "%file%"
