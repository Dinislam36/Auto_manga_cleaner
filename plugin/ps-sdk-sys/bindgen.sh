#!/bin/bash
set -eu

XWIN_CACHE=/home/dcnick3/.cache/cargo-xwin/xwin/
PSSDK=/home/dcnick3/projects/mangai/adobe_photoshop_sdk_2021_win_v1

bindgen ps-api.h -o src/generated.rs \
    --allowlist-file "$PSSDK/pluginsdk/photoshopapi/photoshop/PIFilter.h" \
    --allowlist-file "$PSSDK/pluginsdk/photoshopapi/photoshop/PIGeneral.h" \
    -- -target x86_64-pc-windows-msvc -DWIN32=1 \
    -isystem "$XWIN_CACHE/sdk/include/um/" \
    -isystem "$XWIN_CACHE/sdk/include/shared/" \
    -isystem "$XWIN_CACHE/sdk/include/ucrt/" \
    -isystem "$XWIN_CACHE/crt/include/" \
    -isystem "$PSSDK/pluginsdk/photoshopapi/pica_sp/" \
    -isystem "$PSSDK/pluginsdk/photoshopapi/photoshop/"
