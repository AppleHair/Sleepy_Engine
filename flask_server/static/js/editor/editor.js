//////////////////////////////////////////////////////
//          2D GAME ENGINE - WEB EDITOR APP         //
//////////////////////////////////////////////////////

// This file is the entry point of the editor's JavaScript codebase.
// Here many of the editor's primary behaviors are defined, and the
// editor's interaction with the server and the project file is done.

//////////////////////////////
//          Imports         //

// We import the DOM modules' wrapper
import documentInteractionSetup, * as DOM from "./DOMcontrol.js";

// We import the SQL Control module
import initSQLite, * as SQLITE from "./SQLcontrol.js";

/////////////////////////////////////////
//           Repeatedly Used           //

// the current game test window
let gameTestWindow = undefined;

// Lists of element and asset rowids
// that need to be loaded or updated
// in the game test window.
let assetsToLoad = [];
let elementsToLoad = [];

self.setupTestWindow = function(new_window) {
    // Get all the asset rowids
    // and add them to the assetsToLoad
    // array, which will make the game
    // test load all the assets when it
    // starts, and update them when they
    // get changed in the editor.
    let allAssets = [];
    SQLITE.forEachInTable(project, "asset", (row) => {
        allAssets.push([row["rowid"], row["type"]]);
    });
    assetsToLoad = allAssets;

    // The same thing applies to elements
    let allElements = [];
    SQLITE.forEachInTable(project, "element", (row) => {
        allElements.push([row["rowid"], row["type"]]);
    });
    elementsToLoad = allElements;

    // Gives the data blob of the game's icon
    // new_window.self.getMetadataIcon = function() {
    //     return getBlob(3);
    // };
    // Gives the script text of the state manager
    new_window.self.getMetadataScript = function() {
        return getBlob(2, true);
    };
    // Gives the data blob of an
    // asset with the given rowid
    new_window.self.getAssetData = function(rowid) {
        return getBlob(SQLITE.getRow(project, "asset", rowid).data);
    };
    // Gives the script text of an
    // element with the given rowid
    new_window.self.getElementScript = function(rowid) {
        return getBlob(SQLITE.getRow(project, "element", rowid).script, true);
    };
    // Gives the config text of the state manager
    new_window.self.getMetadataConfig = function() {
        return getBlob(1, true);
    };
    // Gives the config text of an
    // asset with the given rowid
    new_window.self.getAssetConfig = function(rowid) {
        return getBlob(SQLITE.getRow(project, "asset", rowid).config, true);
    };
    // Gives the config text of an
    // element with the given rowid
    new_window.self.getElementConfig = function(rowid) {
        return getBlob(SQLITE.getRow(project, "element", rowid).config, true);
    };
    // Gives the rowid of the first
    // element with the given name
    new_window.self.getElementID = function(name) {
        let stmt = project.prepare(`SELECT rowid FROM element WHERE name=?;`, [name]);
        stmt.step();
        let res = stmt.get();
        stmt.free();
        return res[0];
    };
    // Gives the rowid of the first
    // asset with the given name
    new_window.self.getAssetID = function(name) {
        let stmt = project.prepare(`SELECT rowid FROM asset WHERE name=?;`, [name]);
        stmt.step();
        let res = stmt.get();
        stmt.free();
        return res[0];
    };
    // Gives the name of an
    // element with the given rowid
    new_window.self.getElementName = function(rowid) {
        const result = SQLITE.getRow(project, "element", rowid).name;
        return ((result === undefined) ? "" : result);
    };
    // Gives the name of an
    // asset with the given rowid
    new_window.self.getAssetName = function(rowid) {
        const result = SQLITE.getRow(project, "asset", rowid).name;
        return ((result === undefined) ? "" : result);
    };
    // Gives the type number of an
    // element with the given rowid
    new_window.self.getElementType = function(rowid) {
        return SQLITE.getRow(project, "element", rowid).type;
    };
    // Gives the type number of an
    // asset with the given rowid
    // new_window.self.getAssetType = function(rowid) {
    //     return SQLITE.getRow(project, "asset", rowid).type;
    // };
    // Gives a list of all the asset rowids
    // of assets that need to be loaded or
    // updated in the game test window.
    new_window.self.assetsToLoad = function() {
        let toLoad = assetsToLoad;
        assetsToLoad = [];
        return toLoad;
    };
    // Gives a list of all the element rowids
    // of elements that need to be loaded or
    // updated in the game test window.
    new_window.self.elementsToLoad = function() {
        let toLoad = elementsToLoad;
        elementsToLoad = [];
        return toLoad;
    };

    // if the game test window gets closed
    // or refreshed, we set the gameTestWindow
    // variable to undefined
    new_window.onbeforeunload = () => {
        if (gameTestWindow !== undefined) {
            gameTestWindow = undefined;
        }
    }

    // when the game test window loads
    // we set its title and icon to the
    // project's title and icon
    new_window.onload = (e) => {
        e.target.title = projectName;
        e.target.querySelector("#game-icon").setAttribute("href", document.querySelector("#game-icon-label > img").src);
    };
    // the setup is done, so we set
    // the gameTestWindow variable to
    // the new window
    gameTestWindow = new_window;
}

// if the editor gets closed
// and the game test is open,
// close the game test too
onpagehide = (e) => {
    if (gameTestWindow !== undefined && !e.persisted) {
        gameTestWindow.close();
    }
};

// Encodes test into binary
const textEncoder = new TextEncoder();
// Decodes binary into text
const textDeoder = new TextDecoder();

// This function receives a rowid
// and returns the blob data of the
// blob with that rowid from the project file.
// to can also choose to get the blob as text.
function getBlob(rowid, asText = false) {
    const blob = SQLITE.getRow(project, "blobs", rowid).data;
    return ((asText) ? textDeoder.decode(blob) : blob);
}

// This function loads data from
// the project file into the document
// and overrides previous data in the
// document if there is any.
function loadEditor() {
    // if the game test is open,
    // close it before loading the
    // new project file
    if (gameTestWindow !== undefined) {
        gameTestWindow.close();
    }
    // We empty the explorer item list
    document.querySelectorAll(".explorer > .item-list > *").forEach((item) => item.remove());
    // We reset the material section
    DOM.resetMaterial(document.querySelector(".editor > .material-section").dataset['mode']);
    // We empty the editor tab list
    document.querySelectorAll(".editor > .tab-section > .tab-list > *").forEach((tab) => tab.remove());
    // We select tab-filler
    document.querySelector(".editor > .tab-section > .tab-filler").id = "tabSelected";
    // We set the project name span's innerText to the project name.
    document.getElementById("projectName").innerText = projectName;

    // We iterate on the names of the
    // tables we want to load items from.
    for (let table of ["folder", "element", "asset"]) {
        // We add every row from these tables to
        // the explorer item list through the DOM.
        SQLITE.forEachInTable(project, table, (row) => DOM.addItem(table, row, types));
    }

    // we load the new icon into the editor
    loadGameIcon(new Blob([SQLITE.getRow(project, "blobs", 3)['data']]));
    // when the user clicks on the icon
    // label (which turns out to be the
    // image in practice), we set the value
    // of the icon input to an empty string
    // to make sure any new icon the user
    // picks will override the current one,
    // even if it's using the same file (forces
    // the browser to replace the icon image).
    document.querySelector("#game-icon-label").onclick = () => {
        document.querySelector("#game-icon-input").value = '';
    };
    // when the user picks a new icon
    // for the game, we load it into
    // the editor and the project file
    document.querySelector("#game-icon-input").oninput = (event) => {
        event.target.files[0].arrayBuffer().then((result) => {
            SQLITE.updateRowValue(project, "blobs", 3,
                'data', new Uint8Array(result));
            loadGameIcon(event.target.files[0]);
        });
    }
}

// This function will take a blob
// which should contain an icon file,
// and load it into the editor as the
// game's icon.
function loadGameIcon(blob) {
    // get the game icon image HTML element
    const gameIconImg = document.querySelector("#game-icon-label > img");
    // we create a new URL for the icon
    // and replace the current icon with it
    if (gameIconImg.src !== undefined) {
        window.URL.revokeObjectURL(gameIconImg.src);
    }
    gameIconImg.src = window.URL.createObjectURL(blob);
}

// This function receives an
// item HTML element, and deletes
// it from the project file
// and the document if the
// user confirms the action
// through the pop-up window
function itemDeletion(item) {
    // We define a message for the user
    let message = `The "${item.getAttribute("name")}" ${item.className} will be deleted permanently.
    
    Please confirm.`
    // if the item is a folder
    if (item.className == "folder") {
        // We make the message fit the 
        // action of deleting a folder
        message = `The "${item.getAttribute("name")}" folder and all of its contents will be deleted permanently.
        
        Please confirm.`
    }
    // We open the pop-up window
    // with a message for the user
    DOM.openMessageWindow(message,
        // When the user confirms
        () => {
            // we remove the item and receive the results
            const result = DOM.removeItem(item,itemDeletionValidation);
            // We also switch the
            // material so if the current
            // tab contained the deleted item,
            // it will be replaced with another tab
            DOM.switchMaterial();
            // if any message was received
            // we alert the user with it
            for (let message of result.messages) {
                alert(message);
            }
            // if any item was removed, we
            // delete it from the project file
            for (let entry of result.removed) {
                // if the item is not a folder,
                // we need to remove its blobs too
                if (entry.table != "folder") {
                    // We get the row of the item
                    const row = SQLITE.getRow(project, entry.table, entry.rowid);
                    // We get the rowid of its config
                    // blob and put it in a new array
                    const blobIDs = [row['config']];
                    
                    // if the item is an asset,
                    // We push the rowid of its
                    // data blob into the array
                    if (entry.table == "asset") {
                        blobIDs.push(row['data']);
                    }
                    // if the item is an element,
                    // we push the rowid of its
                    // script blob into the array
                    if (entry.table == "element") {
                        blobIDs.push(row['script']);
                    }
                    
                    // We itrate on the array
                    // and delete every blob 
                    // referenced by the array
                    for (let rowid of blobIDs) {
                        if (rowid != 0) {
                            SQLITE.deleteRow(project, "blobs", rowid);
                        }
                        // if any rowid is zero,
                        // the blob doesn't exist,
                        // so we skip it.
                    }
                }
                SQLITE.deleteRow(project, entry.table, entry.rowid);
            }
        }
    );
}

// This function receives an
// item HTML element, and renames
// it in the project file and
// the document according to 
// the user's input in the 
// pop-up window
function itemRename(item) {
    // We open an input window for
    // the user, where he will be
    // able to edit the item's name 
    DOM.openInputWindow(`Please rename the "${item.getAttribute("name")}" ${item.className}.`, ['name'],
        // When the user confirms
        (results) => {
            // We update the row of the
            // item with the new name
            SQLITE.updateRowValue(project, item.className, item.dataset['tableId'], 'name', results[0]);
            // We set the name attribute
            // of the item to the new name
            item.setAttribute("name", results[0]);
            // We set the innerText of
            // the item to the new name
            item.querySelector(":scope span").innerText = results[0];
            // we get the item's tab if it exists
            const tab = DOM.fromItemToTab(item);
            if (tab === null) {
                return;
            }
            // We set the name attribute
            // of the tab to the new name
            tab.setAttribute("name", results[0]);
            // We set the innerText of
            // the tab to the new name
            tab.querySelector(":scope span").innerText = results[0];
        }, 
        undefined, SQLITE.getRow(project,item.className,item.dataset['tableId'])
    );
}

// This function checks if a certain
// item can be deleted without breaking
// other items which might use it. if 
// the function finds a problem, it'll 
// return a message that should be seen
// by the user which tried to delete the
// item, and otherwise, it'll return null
function itemDeletionValidation(table, rowid, type) {
    // This value will have aditional information
    // we'll add to the message if we find
    // a problem with the item
    let info = "";
    // This value will tell us if a problem
    // was found with the item
    let invalid = false;
    // if the item is a scene element
    if (type == "scene") {
        // get the state manager's config
        let state = JSON.parse(textDeoder.decode(SQLITE.getRow(project, "blobs", 1)['data']));
        // if the scene for deletion is the initial scene
        if (state['initial-scene'] == rowid) {
            // its deletion is invalid
            invalid = true;
            info = "it's the initial scene of the game";
        }
    }
    // if the item is an object element
    if (type == "object") {
        // keep track of the scenes
        // this object is being used in
        let scenes = [];
        for (let scene of project.exec("SELECT name,config FROM element WHERE type=?;", [2])[0].values) {
            let config = JSON.parse(textDeoder.decode(SQLITE.getRow(project, "blobs", scene[1])['data']));
            for (let inst of config['object-instances']) {
                if (inst['id'] == rowid) {
                    scenes.push(scene[0]);
                    break;
                }
            }
        }
        // if the object is being used in any scene
        if (scenes.length > 0) {
            // its deletion is invalid
            invalid = true;
            info = `it's being used in the following scenes: ${scenes.join(", ")}`;
        }
    }
    // if the item is an asset
    if (table == "asset") {
        // keep track of the objects
        // this asset is being used in
        let objects = [];
        for (let object of project.exec("SELECT name,config FROM element WHERE type=?;", [1])[0].values) {
            let config = JSON.parse(textDeoder.decode(SQLITE.getRow(project, "blobs", object[1])['data']));
            for (let assetRowId of config[`${type}s`]) {
                if (assetRowId == rowid) {
                    objects.push(object[0]);
                    break;
                }
            }
        }
        // if the asset is being used in any object
        if (objects.length > 0) {
            // its deletion is invalid
            invalid = true;
            info = `it's being used in the following objects: ${objects.join(", ")}`;
        }
    }
    // if the item's deletion is invalid
    // we return a message for the user
    // which tried to delete it, and
    // otherwise, we return null
    return invalid ? `Failed Requirement:
The ${type} ${table} you just tried to delete is still being used in this project. (${info})
please remove its use from the project before you delete it.` : null;
}

//////////////////////////////////////////////////////////
//          Database file related definitions           //

// This database stores data
// related to the types of items
// that can be created in this engine.
let typesDB;
// This is a map that helps other
// control modules find the type
// names of the corresponding IDs
// stores in the project file.
let types;
// We create a ServerDBRequest
// object and use it to define
// the typesDB database object
// and other related variables.
new SQLITE.ServerDBRequest(
    // We give it the url
    // to the database file.
    '/static/base-files/type.sqlite',
    // We give it this function
    (db) => {
        // We assign the db
        // reference to typesDB.
        typesDB = db;
        // We create the types map
        // using the typesDB database.
        types = new Map([
            ["element", SQLITE.getMap(typesDB, "elementType", "rowid", "name")],
            ["asset", SQLITE.getMap(typesDB, "assetType", "rowid", "name")]
        ]);
    }
);

// This database is the current
// project file. It's defined as
// a new project template when the
// page loads.
let project;
// This variable represents the
// loaded project's name, and will
// change when the user loads a new
// project file to the editor
let projectName = "new project";
// We create a ServerDBRequest
// object and use it to define
// the project database object.
new SQLITE.ServerDBRequest(
    // We give it the url
    // to the database file.
    '/static/base-files/new-project.sqlite',
    // We give it this function
    (db) => {
        // We assign the db
        // reference to project.
        project = db;
    }
);

//////////////////////////////////////
//          Project Loading         //

// We initialize SQLite, which
// loads all the server databases
// which were requested, and we also
// set up the items in the explorer
// when the databases finish loading.
initSQLite().then((loaded) => {
    // if the server databases didn't load.
    if (!loaded) {
        alert("Couldn't load your project file");
        return;
    }

    // We load the loaded
    // project file into
    // the document
    loadEditor();

    // We set up the document
    // interaction behavior, and
    // define many action functions
    documentInteractionSetup({

            ////////////////////////////////////////////////
            //          Drop Down Menu functions          //

            // This function will let the
            // user load his own project
            // file from his hard disk 
            // drive to the editor.
            'load-project': () => {
                // We get the document's load "file input" HTML element
                const fileInput = document.querySelector("#project-load");
                // We reset the value of the
                // input HTML element to make the
                // file reload even if it's
                // already selected
                fileInput.value = '';
                // We show the file picker
                // of the HTML element to the user
                fileInput.showPicker();
                // When the user picks a file
                fileInput.oninput = () => {
                    // We read the file the 
                    // user picked as an array buffer
                    fileInput.files[0].arrayBuffer().then((result) => {
                        // we convert the file's array buffer into
                        // an array of unsigned 8-bit integers. We
                        // use it to create a new database object,
                        // and replace it with the currently stored
                        // project file.
                        project = SQLITE.loadFromUint8Array(new Uint8Array(result));
                        // We set the file's name
                        // to the project's name
                        projectName = fileInput.files[0].name;
                        // We remove the file extension 
                        // from the project name
                        projectName = projectName.slice(0,projectName.lastIndexOf('.'));
                        // We load the new project 
                        // file into the editor
                        loadEditor();
                    });
                };
            },
            // This function will let the user
            // download his project file to his
            // hard disk drive.
            'save-project': () => {
                // We get the document's save "a" HTML element
                const a = document.querySelector("#project-save");
                // We create a new URL for the project
                // file and give it to the a HTML element.
                a.href = URL.createObjectURL(new Blob([project.export()]));
                // We define the a HTML element as a download link and
                // tell it to download the file with the name of the project.
                a.download = projectName + ".sqlite";
                // We set the target to _black, which will
                // make the download happen on a new window/tab.
                a.target = '_blank';
                // We "click" the a HTML element to tell the
                // browser we want to download the file.
                a.click();
                // We revoke the URL after the download.
                URL.revokeObjectURL(a.href);
            },
            // This function will let the user
            // open a new game test window, which
            // will replace the current one if it's
            // already open.
            'test-game': () => {
                // close the current game
                // test window if it's open
                if (gameTestWindow !== undefined) {
                    gameTestWindow.close();
                    gameTestWindow = undefined;
                }
                // open a new game test window
                window.open("/game-test", "GAME TEST");
            },
            // This function will let the user
            // export his game to an HTML Archive
            // (a zip with an index.html inside).
            'export-game': async () => {
                // We get the document's save "a" HTML element
                const a = document.querySelector("#export-save");
                // We create a FormData object
                let data = new FormData();
                // We append the project file into the form data object
                data.append("gameData", new Blob([project.export()]))
                // We request an game export from the server,
                // create a new URL for it and give it to the a HTML element.
                a.href = URL.createObjectURL(await fetch('/export', { method: "POST", body: data }).then((res) => res.blob()));
                // We define the a HTML element as a download link and
                // tell it to download the file with the name of the project.
                a.download = projectName + ".zip";
                // We set the target to _black, which will
                // make the download happen on a new window/tab.
                a.target = '_blank';
                // We "click" the a HTML element to tell the
                // browser we want to download the file.
                a.click();
                // We revoke the URL after the download.
                URL.revokeObjectURL(a.href);
            }
        }, {

            //////////////////////////////////////////////
            //          Context Menu functions          //

            // This function lets the user
            // create a new folder in a certain
            // container, which can be the item
            // list or another folder.
            'new-folder': (container) => {
                // We define a message for the user
                // which fits the action of adding
                // a folder to the item list HTML element
                let message = `The new folder will be added to the base folder.
                
                Please define the new folder:`;
                // if the container is a folder
                if (container.className == "folder") {
                    // We make the message fit the 
                    // action of adding a folder
                    // to a folder
                    message = `The new folder will be added to the "${container.getAttribute("name")}" folder.
                    
                    Please define the new folder:`;
                }
                // We open an input window for
                // the user, where he will be
                // able to define the basic 
                // attributes of the folder
                DOM.openInputWindow(message, ['name','color'],
                    // When the user confirms
                    (results) => {
                        // We push the container's 
                        // rowid to the results array
                        results.push(container.dataset['tableId']);
                        // We use the results array to add the folder
                        // to the project file and the document
                        DOM.addItem("folder", SQLITE.addRow(project, "folder", results), types, true);
                    }
                );
            },

            // This function lets the user
            // create a new element in a certain
            // container, which can be the item
            // list or a folder.
            'new-element': (container) => {
                // We define a message for the user
                // which fits the action of adding
                // an element to the item list HTML element
                let message = `The new element will be added to the base folder.
                
                Please define the new element:`;
                // if the container is a folder
                if (container.className == "folder") {
                    // We make the message fit the 
                    // action of adding an element to
                    // a folder
                    message = `The new element will be added to the "${container.getAttribute("name")}" folder.
                    
                    Please define the new element:`;
                }
                // We open an input window for
                // the user, where he will be
                // able to define the basic 
                // attributes of the element
                DOM.openInputWindow(message, ['name','type'],
                    // When the user confirms
                    (results) => {
                        // We create a new config file for the element
                        let blobConfig = SQLITE.addRow(project, "blobs", [
                            SQLITE.getRow(typesDB,"blobs",
                                SQLITE.getRow(typesDB,"elementType",results[1])['baseConfig']
                                )['data']
                            ]);
                        // We create a new script file for the element
                        let blobScript = SQLITE.addRow(project, "blobs", [
                            SQLITE.getRow(typesDB,"blobs",
                                SQLITE.getRow(typesDB,"elementType",results[1])['baseScript']
                                )['data']
                            ]);

                        // We push the container's rowid, 
                        // the config blob's rowid and the
                        // script blob's rowid into the results array
                        results.push(container.dataset['tableId'], blobConfig['rowid'], blobScript['rowid']);
                        //
                        let newElementRow = SQLITE.addRow(project, "element", results);
                        // We use the results array to add the element
                        // to the project file and the document
                        DOM.addItem("element", newElementRow, types, true);
                        //
                        elementsToLoad.push([newElementRow['rowid'], newElementRow['type']]);
                        // We also switch the
                        // material so the user
                        // could see the contents
                        // of the new item they created
                        DOM.switchMaterial();
                    }, 
                    types.get("element")
                );
            },

            // This function lets the user
            // create a new asset in a certain
            // container, which can be the item
            // list or a folder.
            'new-asset': (container) => {
                // We define a message for the user
                // which fits the action of adding
                // an asset to the item list HTML element
                let message = `The new asset will be added to the base folder.
                
                Please define the new asset:`;
                // if the container is a folder
                if (container.className == "folder") {
                    // We make the message fit the 
                    // action of adding an asset to
                    // a folder
                    message = `The new asset will be added to the "${container.getAttribute("name")}" folder.
                    
                    Please define the new asset:`;
                }
                // We open an input window for
                // the user, where he will be
                // able to define the basic 
                // attributes of the asset
                DOM.openInputWindow(message, ['name','type', 'data'],
                    // When the user confirms
                    async (results) => {
                        // We pop the asset file 
                        // out of the results array
                        const assetFile = results.pop();
                        // We get the file extention from the file's name
                        const fileExten = assetFile.name.substring(assetFile.name.lastIndexOf('.') + 1);
                        // if the file format doesn't correspond with the asset type
                        if (fileExten != SQLITE.getRow(typesDB,"assetType",results[1])['dataType']) {
                            // we alert the user and return
                            // (the window won't close)
                            alert(`Failed Requirement:
                                Your asset couldn't be created,
                                because the file format which
                                was used didn't correspond with
                                the asset type which was chosen.`);
                            return;
                        }

                        // when the asset file gets
                        // loaded by the file reader
                        const buffer = await assetFile.arrayBuffer();
                        // We add the asset file to the blobs table
                        let blobData = SQLITE.addRow(project, "blobs", [new Uint8Array(buffer)]);

                        // We create a new config file for the asset
                        let blobConfig = SQLITE.addRow(project, "blobs", [
                            SQLITE.getRow(typesDB,"blobs",
                                SQLITE.getRow(typesDB,"assetType",results[1])['baseConfig']
                                )['data']
                            ]
                        );

                        // We push the container's rowid,
                        // the config blob's rowid and the
                        // data blob's rowid into the results array
                        results.push(container.dataset['tableId'], blobConfig['rowid'], blobData['rowid']);
                        //
                        let newAssetRow = SQLITE.addRow(project, "asset", results);
                        // We use the results array to add the asset
                        // to the project file and the document
                        DOM.addItem("asset", newAssetRow, types, true);
                        //
                        assetsToLoad.push([newAssetRow['rowid'], newAssetRow['type']]);
                        // We also switch the
                        // material so the user
                        // could see the contents
                        // of the new item they created
                        DOM.switchMaterial();
                    }, 
                    types.get("asset")
                );
            },

            // This function lets the
            // user recolor a folder
            'recolor-folder': (folder) => {
                // We open an input window
                // for the user, where he'll
                // be able to edit the folder's color
                DOM.openInputWindow(`Please re-color the "${folder.getAttribute("name")}" folder:`, ['color'],
                    // When the user confirms
                    (results) => {
                        // We update the row of the
                        // item with the new hex color value
                        SQLITE.updateRowValue(project, "folder", folder.dataset['tableId'], 'color', results[0]);
                        // We set the --folder-color
                        // css property of the item 
                        // to the new hex color value
                        folder.setAttribute("style", `--folder-color: ${results[0]};`);
                    }, 
                    undefined, SQLITE.getRow(project,"folder",folder.dataset['tableId'])
                );
            },
            // This function lets the
            // user rename a folder
            'rename-folder': (folder) => {
                itemRename(folder);
            },
            // This function lets the
            // user rename a element
            'rename-element': (element) => {
                itemRename(element);
            },
            // This function lets the
            // user rename a asset
            'rename-asset': (asset) => {
                itemRename(asset);
            },
            // This function lets the
            // user delete a folder
            'delete-folder': (folder) => {
                itemDeletion(folder);
            },
            // This function lets the
            // user delete a element
            'delete-element': (element) => {
                itemDeletion(element);
            },
            // This function lets the
            // user delete a asset
            'delete-asset': (asset) => {
                itemDeletion(asset);
            }
        }, {

            //////////////////////////////////////////////////
            //          Material Switch functions           //

            // This function will be called
            // when the user switches the
            // material to a config of an item.
            // Its job is to load the config of
            // the item from the project file
            // and let the material section use
            // it to display the item's config
            // to the user.
            'config': (table, rowid) => {
                // get the rowid of the config in the 'blobs' table
                const blobid = (table == 'metadata') ? 1 : SQLITE.getRow(project, table, rowid)['config'];
                // get the config in text from the 'blobs' table
                const text = textDeoder.decode(SQLITE.getRow(project, "blobs", blobid)['data']);
                // parse the config text into a JSON object
                // and return it with the blob rowid
                return {JSON: JSON.parse(text), blobID: blobid};
            },

            // This function does the same
            // thing as the previous one, but
            // for the script of an item.
            'script': (table, rowid) => {
                let blobid = (table == 'metadata') ? 2 : SQLITE.getRow(project, table, rowid)['script'];
                const text = textDeoder.decode(SQLITE.getRow(project, "blobs", blobid)['data']);
                // because it's a script, we
                // return the text itself
                // without parsing it
                return {text: text, rowid: blobid};
            }
        }, {

            //////////////////////////////////////////////////
            //          Material Change functions           //

            // This function will be called
            // when the user changes a value
            // in the config of an item.
            'config': (configInfo) => {
                // get the rowid of the config in the 'blobs' table
                const blobid = configInfo.blob;
                // get the edited config in text
                const blob = textEncoder.encode(JSON.stringify(configInfo.JSON));
                // update the config in the 'blobs' table
                SQLITE.updateRowValue(project, "blobs", blobid, "data", blob);
                // get the table and rowid of the config's
                // associated item and add it to the list
                // of items that need to be updated in the
                // game test window, if they aren't already
                // in the list
                const curTab = document.querySelector("#tabSelected");
                if (curTab.dataset['table'] == "metadata") {
                    return;
                }
                if (curTab.dataset['table'] == "element") {
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "element", Number(curTab.dataset['tableId']))['type']];
                    if (elementsToLoad.indexOf(toLoad) == -1) {
                        elementsToLoad.push(toLoad);
                    }
                    return;
                }
                if (curTab.dataset['table'] == "asset") {
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "asset", Number(curTab.dataset['tableId']))['type']];
                    if (assetsToLoad.indexOf(toLoad) == -1) {
                        assetsToLoad.push(toLoad);
                    }
                    return;
                }
            },

            // This function does the same
            // thing as the previous one, but
            // for the script of an item.
            'script': (text, blobid) => {
                // update the script in the 'blobs' table
                SQLITE.updateRowValue(project, "blobs", blobid, "data", textEncoder.encode(text));
                // get the table and rowid of the script's
                // associated element and add it to the list
                // of elements that need to be updated in the
                // game test window, if they aren't already
                // in the list
                const curTab = document.querySelector("#tabSelected");
                if (curTab.dataset['table'] == "metadata") {
                    return;
                }
                if (curTab.dataset['table'] == "element") {
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "element", Number(curTab.dataset['tableId']))['type']];
                    if (elementsToLoad.indexOf(toLoad) == -1) {
                        elementsToLoad.push(toLoad);
                    }
                    return;
                }
            }
        }, {
           
            ///////////////////////////////////////////////////////////
            //          Before Material Change functions             //

            // This function will be called
            // "before" the user changes a value
            // in the config of an item. It will
            // get the current value of the specific
            // input element that the user edited and
            // the previous value that's in the config.
            // the purpose of this function is to filter
            // the user's input and make sure it's valid.
            'config-input': (inputElement, prvValue) => {
                // get the changed value from the input element
                let curValue = (inputElement.type == "number" || inputElement.type == "range") ?
                    Number(inputElement.value) : inputElement.value;
                // if the input is an element or asset rowid,
                // we need to make sure the rowid is valid
                if (inputElement.className == "element-user" || inputElement.className == "asset-user") {
                    // determine the table and type of the rowid
                    let type = 1;
                    let table = "element";
                    if (inputElement.className == "element-user") {
                        if (inputElement.name == "initial-scene") {
                            type = 2;
                        }
                    }
                    if (inputElement.className == "asset-user") {
                        table = "asset";
                        const assetlist = inputElement.parentNode.parentNode.parentNode.getAttribute("name");
                        type = 3;
                        if (assetlist == "sprites") {
                            type = 1;
                        } else if (assetlist == "audios") {
                            type = 2;
                        }
                    }
                    // check for the closest valid rowid
                    // in relation to the previous value
                    // and from the direction of the change
                    let res = null;
                    if (curValue == prvValue) {
                        res = undefined;
                    } else if (curValue < prvValue) {
                        res = project.exec(`SELECT rowid,name FROM ${table} WHERE rowid<? AND type=?;`, [prvValue,type])[0];
                        if (res === undefined && table == 'asset' && type == 1) {
                            res = {"values": [[0, 'clean']]}
                        } else if (res !== undefined) {
                            res.values[0] = res.values[res.values.length-1];
                        }
                    } else {
                        res = project.exec(`SELECT rowid,name FROM ${table} WHERE rowid>? AND type=?;`, [prvValue,type])[0];
                    }
                    if (res !== undefined) {
                        // for now, the name and rowid/index
                        // will be printed to the console
                        // every time the user changes the
                        // rowid/index of an element, asset
                        // or layer. this will help them
                        // identify the rowid/index they're
                        // looking for.
                        console.log(res.values[0]);
                    }
                    return (res !== undefined) ? res.values[0][0] : prvValue;
                }
                // if the input is a layer index,
                // we need to make sure the index is in bounds
                if (inputElement.className == "layer-user") {
                    // if the current value is -1
                    // or smaller, we return -1
                    if (curValue <= -1) {
                        console.log([-1, "None"]);
                        return -1;
                    }
                    // find the layers list and iterate on it.
                    // In the iteration, we'll find the last
                    // layer index and name, and the layer
                    // index and name associalted with the
                    // current value. 
                    const layersList = document.querySelector("#scene-config > [name='layers']").
                        querySelectorAll(":scope > li [name]");
                    let name = "";
                    let lastName = "";
                    layersList.forEach((layer) => {
                        if (curValue == Number(layer.getAttribute("name"))) {
                            name = layer.value;
                        }
                        if (layersList.length-1 == Number(layer.getAttribute("name"))) {
                            lastName = layer.value;
                        }
                    });
                    // if the current value is bigger
                    // than the last layer index, we
                    // return the last layer index
                    if (layersList.length-1 < curValue) {
                        console.log([layersList.length-1, lastName]);
                        return layersList.length-1;
                    }
                    // otherwise, we return the current value
                    console.log([curValue, name]);
                }
                return curValue;
            },

            // This function will be called
            // "before" the user removes a member
            // from a list in the config of an item.
            // This function will determine if the
            // user is allowed to remove the member
            // from the list.
            'config-minus': (li) => {
                // Make sure the user can't remove
                // a layer which is being used by
                // an object instance
                if (li.parentNode.getAttribute("name") == "layers") {
                    let name = li.querySelector(":scope [name]").getAttribute("name");
                    let res = true;
                    document.querySelectorAll('.layer-user:not(.li-template *)').forEach((user) => {
                        res = user.value != name && res;
                    });
                    return res;
                }
                // Make sure the user can't remove
                // anything in any asset's config if
                // the game test window is open
                return gameTestWindow === undefined || !li.matches("#sprite-config *");
            },
            // This function will be called
            // "before" the user adds a member
            // to a list in the config of an item.
            // This function will determine if the
            // user is allowed to add the member
            // to the list.
            'config-plus': (list) => {
                // make sure the user can add an
                // element or asset to a config only if such
                // element or asset exists in the project
                if ((list.getAttribute("name") == "object-instances" &&
                project.exec(`SELECT rowid FROM element WHERE type=?;`, [1])[0] === undefined) ||
                (list.getAttribute("name") == "sprites" &&
                project.exec(`SELECT rowid FROM asset WHERE type=?;`, [1])[0] === undefined) ||
                (list.getAttribute("name") == "audios" &&
                project.exec(`SELECT rowid FROM asset WHERE type=?;`, [2])[0] === undefined) ||
                (list.getAttribute("name") == "fonts" &&
                project.exec(`SELECT rowid FROM asset WHERE type=?;`, [3])[0] === undefined)) {
                    return false;
                }
                return true;
            }
        }
    );
});