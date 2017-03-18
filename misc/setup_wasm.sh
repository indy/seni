source ~/code/wasm/emsdk/emsdk_env.sh

export PATH=~/code/wasm/emsdk/emscripten/incoming:"${PATH}"

mkdir build_wasm
cp misc/html_template/seni.html build_wasm/.
