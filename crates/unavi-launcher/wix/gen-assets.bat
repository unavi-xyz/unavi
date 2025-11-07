@echo off
setlocal enabledelayedexpansion

set "file=assets.wxs"
set "tempfile=tmp.wxs"

heat dir ../assets -cg AssetsGroup -dr AssetsFolder -ag -sfrag -out %file%

:: Replace SourceDir with workspace-relative path.
(for /f "usebackq delims=" %%a in ("%file%") do (
    set "line=%%a"
    set "line=!line:SourceDir=crates\unavi-launcher\assets!"
    echo !line!
)) > "%tempfile%"

move /y "%tempfile%" "%file%"
