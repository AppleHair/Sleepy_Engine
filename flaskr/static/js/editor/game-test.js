
//
import initEngine, * as wasmEngine from "./engine-code/game_engine.js";

//
initEngine().then(() => {
    wasmEngine.handle_game();
});