
//
opener.self.setupTestWindow(window);

//
import initEngine, * as wasmEngine from "../engine-core/game_engine.js";

//
let update_handle;

//
document.addEventListener("contextmenu", (event) => event.preventDefault(), true);
document.addEventListener("keydown", (event) => event.preventDefault(), true);
document.addEventListener("keyup", (event) => event.preventDefault(), true);

//
initEngine().then(() => {
    update_handle = wasmEngine.handle_game();
});