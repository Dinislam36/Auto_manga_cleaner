#!/bin/bash

# TODO: rewrite in rust

set -eu

XWIN_CACHE=/home/dcnick3/.cache/cargo-xwin/xwin
PSSDK=/home/dcnick3/projects/mangai/adobe_photoshop_sdk_2021_win_v1

clang -x c -P -E -target x86_64-pc-windows-msvc -DWIN32=1 \
  -isystem "$XWIN_CACHE/sdk/include/um/" \
  -isystem "$XWIN_CACHE/sdk/include/shared/" \
  -isystem "$XWIN_CACHE/sdk/include/ucrt/" \
  -isystem "$XWIN_CACHE/crt/include/" \
  -isystem "$PSSDK/pluginsdk/photoshopapi/pica_sp/" \
  -isystem "$PSSDK/pluginsdk/photoshopapi/photoshop/" \
  -isystem "$PSSDK/pluginsdk/samplecode/common/includes/" \
  -isystem "$PSSDK/pluginsdk/samplecode/common/resources/" \
  MangaiClean.r -o MangaiClean.rr

WINEDEBUG=-all wine "$PSSDK/pluginsdk/samplecode/resources/Cnvtpipl.exe" MangaiClean.rr MangaiClean.rc

iconv -f iso-8859-1 -t utf-8 MangaiClean.rc -o MangaiClean_utf8.rc

x86_64-w64-mingw32-windres MangaiClean_utf8.rc MangaiClean.o
llvm-lib MangaiClean.o