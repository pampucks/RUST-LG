@echo off

echo [1/8] Authenticating SSH Key with device...
:: This pipes your passphrase directly into the command automatically
(echo 0BF92D) | call ares-novacom -d 303 --getkey

echo [2/8] Uninstalling existing app...
call ares-install -r com.lg.app.signage.dev -d 303

echo [3/8] Cleaning up old package files...
if exist com.lg.app.signage.dev del /Q com.lg.app.signage.dev
if exist *.ipk del /Q *.ipk

echo [4/8] Building WASM...
call wasm-pack build --target web

echo [5/8] Inlining WASM bytes...
node inline-wasm.js

echo [6/8] Copying pkg to webos-dist...
xcopy /E /Y pkg\* webos-dist\pkg\

echo [7/8] Packaging webOS app...
call ares-package webos-dist services

echo [8/8] Installing new app...
call ares-install com.lg.app.signage.dev_1.0.0_all.ipk -d 303

echo ---------------------------------------
echo Launching inspector...
echo Use Ctrl + C in this window to stop.
echo ---------------------------------------
call ares-inspect com.lg.app.signage.dev -d 303

:HOLD_TERMINAL
timeout /t 3600 /nobreak >nul
goto HOLD_TERMINAL