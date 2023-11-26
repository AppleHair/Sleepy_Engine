
//
import initEngine, * as wasmEngine from "./game_engine.js";

// This is a SqlJs object, and its
// purpose is to create Database objects.
// It's defined using the initSqlJs method
// in the init function.
let sqlite;
// This object references the game's
// data sqlite file, and uses sql.js's
// Database object API to interact with it.
let gameData;
// Lists of element and asset rowids
// that need to be loaded by the engine core.
let assetsToLoad = [];
let elementsToLoad = [];

// Decodes binary into text
const textDeoder = new TextDecoder();

// Prevents the browser from interfering
// with the game's key and mouse events.
document.addEventListener("contextmenu", (event) => event.preventDefault(), true);
document.addEventListener("keydown", (event) => event.preventDefault(), true);
document.addEventListener("keyup", (event) => event.preventDefault(), true);

// This function receives a database
// object, a table name, and a callback
// function. Using these arguments, it
// iterates on each row in the requested
// table, and calls the callback function
// every iteration, giving it the row's object
// as the first argument.
function forEachInTable(table, callback) {
    gameData.each(`SELECT rowid, * FROM ${table};`, [], callback);
}

// This function receives a database
// object, a table name, and a rowid.
// Using these arguments, it finds the
// correct row from the requested table,
// which has the requested rowid, and
// returns it as an object.
function getRow(table, rowid) {
    let stmt = gameData.prepare(`SELECT rowid, * FROM ${table} WHERE rowid=?;`, [rowid]);
    stmt.step();
    let res = stmt.getAsObject();
    stmt.free();
    return res;
}

// This function receives a rowid
// and returns the blob data of the
// blob with that rowid from the project file.
// to can also choose to get the blob as text.
function getBlob(rowid, asText = false) {
    const blob = getRow("blobs", rowid).data;
    return ((asText) ? textDeoder.decode(blob) : blob);
}

// Gives the data blob of the game's icon
// globalThis.getMetadataIcon = function() {
//     return getBlob(3);
// };
// Gives the script text of the state manager
globalThis.getMetadataScript = function() {
    return getBlob(2, true);
};
// Gives the data blob of an
// asset with the given rowid
globalThis.getAssetData = function(rowid) {
    return getBlob(getRow("asset", rowid).data);
};
// Gives the script text of an
// element with the given rowid
globalThis.getElementScript = function(rowid) {
    return getBlob(getRow("element", rowid).script, true);
};
// Gives the config text of the state manager
globalThis.getMetadataConfig = function() {
    return getBlob(1, true);
};
// Gives the config text of an
// asset with the given rowid
globalThis.getAssetConfig = function(rowid) {
    return getBlob(getRow("asset", rowid).config, true);
};
// Gives the config text of an
// element with the given rowid
globalThis.getElementConfig = function(rowid) {
    return getBlob(getRow("element", rowid).config, true);
};
// Gives the rowid of the first
// element with the given name
globalThis.getElementID = function(name) {
    let stmt = gameData.prepare(`SELECT rowid FROM element WHERE name=?;`, [name]);
    stmt.step();
    let res = stmt.get();
    stmt.free();
    return res[0];
};
// Gives the rowid of the first
// asset with the given name
globalThis.getAssetID = function(name) {
    let stmt = gameData.prepare(`SELECT rowid FROM asset WHERE name=?;`, [name]);
    stmt.step();
    let res = stmt.get();
    stmt.free();
    return res[0];
};
// Gives the name of an
// element with the given rowid
globalThis.getElementName = function(rowid) {
    const result = getRow("element", rowid).name;
    return ((result === undefined) ? "" : result);
};
// Gives the name of an
// asset with the given rowid
globalThis.getAssetName = function(rowid) {
    const result = getRow("asset", rowid).name;
    return ((result === undefined) ? "" : result);
};
// Gives the type number of an
// element with the given rowid
globalThis.getElementType = function(rowid) {
    return getRow("element", rowid).type;
};
// // Gives the type number of an
// // asset with the given rowid
// globalThis.getAssetType = function(rowid) {
//     return getRow("asset", rowid).type;
// };
// Gives a list of all the asset rowids
// of assets that need to be loaded or
// updated in the game test window.
globalThis.assetsToLoad = function() {
    let toLoad = assetsToLoad;
    assetsToLoad = [];
    return toLoad;
};
// Gives a list of all the element rowids
// of elements that need to be loaded or
// updated in the game test window.
globalThis.elementsToLoad = function() {
    let toLoad = elementsToLoad;
    elementsToLoad = [];
    return toLoad;
};

// this function should be used to initiate
// all SQLite database loading functionality
// on the script, and load the game data sqlite
// file
async function init() {
    // If the sqlite object is already
    // defined, we won't want to execute
    // this function again.
    if (sqlite !== undefined) {
        // We return false to indicate that
        // the server databases didn't load.
        return false;
    }
    // SqlJsConfig object description: https://sql.js.org/documentation/global.html#SqlJsConfig

    // We use the initSqlJs to initialize the
    // SqlJs object, passing a SqlJsConfig object
    // into it, with the correct path to the .wasm file.
    sqlite = await initSqlJs({locateFile: (file) => `./${file}`});
    // We use fetch to load the database file object.
    // We then get the file's binary data buffer and convert it into
    // an array of unsigned 8-bit integers. We then give the new
    // array to the Database object constructor, and it creates
    // a new Database object for the game's data.
    gameData = new sqlite.Database(new Uint8Array(await fetch("./data.sqlite").then((res) => {
        if (!res.ok) {
            throw new Error("GAME LOAD ERROR: 'data.sqlite' not found.");
        }
        return res.arrayBuffer();
    })));
}

init().then(() => {
    // Get all the asset rowids
    // and add them to the assetsToLoad
    // array, which will make the engine
    // core load all the assets when it
    // starts, and update them when they
    // get changed in the editor.
    forEachInTable("asset", (row) => {
        assetsToLoad.push([row["rowid"], row["type"]]);
    });
    // The same thing applies to elements
    forEachInTable("element", (row) => {
        elementsToLoad.push([row["rowid"], row["type"]]);
    });

    // initialize the engine core
    // and start the game
    initEngine();
})