
//
import initEngine, * as wasmEngine from "./game_engine.js";

//
let sqlite;
//
let gameData;
//
let assetsToLoad = [];
//
let elementsToLoad = [];

//
const textDeoder = new TextDecoder();

//
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

//
function getBlob(rowid, asText = false) {
    const blob = getRow("blobs", rowid).data;
    return ((asText) ? textDeoder.decode(blob) : blob);
}

//
globalThis.getMetadataIcon = function() {
    return getBlob(3);
};
//
globalThis.getMetadataScript = function() {
    return getBlob(2, true);
};
//
globalThis.getAssetData = function(rowid) {
    return getBlob(getRow("asset", rowid).data);
};
//
globalThis.getElementScript = function(rowid) {
    return getBlob(getRow("element", rowid).script, true);
};
//
globalThis.getMetadataConfig = function() {
    return getBlob(1, true);
};
//
globalThis.getAssetConfig = function(rowid) {
    return getBlob(getRow("asset", rowid).config, true);
};
//
globalThis.getElementConfig = function(rowid) {
    return getBlob(getRow("element", rowid).config, true);
};
//
globalThis.getElementID = function(name) {
    let stmt = gameData.prepare(`SELECT rowid FROM element WHERE name=?;`, [name]);
    stmt.step();
    let res = stmt.get();
    stmt.free();
    return res[0];
};
//
globalThis.getAssetID = function(name) {
    let stmt = gameData.prepare(`SELECT rowid FROM asset WHERE name=?;`, [name]);
    stmt.step();
    let res = stmt.get();
    stmt.free();
    return res[0];
};
//
globalThis.getElementName = function(rowid) {
    const result = getRow("element", rowid).name;
    return ((result === undefined) ? "" : result);
};
//
globalThis.getAssetName = function(rowid) {
    const result = getRow("asset", rowid).name;
    return ((result === undefined) ? "" : result);
};
//
globalThis.getElementType = function(rowid) {
    return getRow("element", rowid).type;
};
//
globalThis.getAssetType = function(rowid) {
    return getRow("asset", rowid).type;
};
//
globalThis.assetsToLoad = function() {
    let toLoad = assetsToLoad;
    assetsToLoad = [];
    return toLoad;
};
//
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

    forEachInTable("asset", (row) => {
        assetsToLoad.push([row["rowid"], row["type"]]);
    });
    forEachInTable("element", (row) => {
        elementsToLoad.push([row["rowid"], row["type"]]);
    });

    //
    initEngine();
})