#!/bin/bash

set -eux

tar -cf - --exclude=target . | ssh wtp 'Desktop\\mangai-plugin\\recv.bat'

#cargo xwin build --target x86_64-pc-windows-msvc --release
#scp target/x86_64-pc-windows-msvc/release/mangai_clean_ps_plugin.dll 'wtp:C:\Program Files\Adobe\Adobe Photoshop 2022\Plug-ins\mangai_clean.8bf'
#ssh wtp PsExec64 -u WTP -p 1234 -i 1 -nobanner '"C:\Program Files\Adobe\Adobe Photoshop 2022\Photoshop.exe"'


