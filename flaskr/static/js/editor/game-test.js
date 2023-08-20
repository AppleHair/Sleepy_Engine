
//
import initEngine, * as wasmEngine from "./engine-code/game_engine.js";

//
let update_handle;

//
initEngine().then(() => {
    update_handle = wasmEngine.handle_game();
});