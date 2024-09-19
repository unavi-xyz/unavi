@echo off
setlocal enabledelayedexpansion

set "search=SourceDir"
set "replace=crates\unavi-app\assets"
set "file=assets.wxs"
set "tempfile=tmp.wxs"

heat dir ../assets -cg AssetsGroup -dr APPLICATIONFOLDER -ag -sfrag -out %file%

:: Replace source
(for /f "usebackq delims=" %%a in ("%file%") do (
    set "line=%%a"
    set "line=!line:%search%=%replace%!"
    echo !line!
)) > "%tempfile%"

move /y "%tempfile%" "%file%"