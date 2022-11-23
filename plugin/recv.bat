REM I HATE WINDOWS I HATE WINDOWS I HATE WINDOWS
cd /d %~dp0
tar -xf -
cargo b --release
copy /y "target\release\mangai_clean_ps_plugin.dll" "C:\Program Files\Adobe\Adobe Photoshop 2022\Plug-ins\mangai_clean.8bf"
