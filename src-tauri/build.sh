# Compila el lector, compila el wrapper para desktop para mac y 
# crea el instalador para mac en la ruta installers/macos
sh ../../lector/build.sh 
cargo tauri build; 
dmgFile=$(echo "$(find ./target/release/bundle/dmg/ -type f -name '*.dmg')")
rm -rf ./installers/macos/
mkdir ./installers/ >/dev/null 2>&1
mkdir ./installers/macos/
cp "$dmgFile" ./installers/macos/ACAM.dmg