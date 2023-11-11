
//
opener.self.setupTestWindow(window);

//
import initEngine, * as wasmEngine from "../engine-core/game_engine.js";

//
document.addEventListener("contextmenu", (event) => event.preventDefault(), true);
//
initEngine();