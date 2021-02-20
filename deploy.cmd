cargo +nightly-i686 build --release
cd ./target/release/
copy /Y "client.dll" "e:/games/GTALauncherGAME3/cef/client.dll"
copy /Y "renderer.exe" "e:/games/GTALauncherGAME3/cef/renderer.exe"
copy /Y "loader.dll" "e:/games/GTALauncherGAME3/cef.asi"
