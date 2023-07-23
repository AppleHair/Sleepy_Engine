cd instance/game-engine
wasm-pack build --target web
cp pkg/game_engine.js ../../flaskr/static/js/editor/engine-code
cp pkg/game_engine_bg.wasm ../../flaskr/static/js/editor/engine-code
cd ../..