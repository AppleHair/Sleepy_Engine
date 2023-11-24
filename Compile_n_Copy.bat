cd game-engine
wasm-pack build --target web
cp pkg/game_engine.js ../flask_server/static/js/engine-core
cp pkg/game_engine_bg.wasm ../flask_server/static/js/engine-core
cd ..