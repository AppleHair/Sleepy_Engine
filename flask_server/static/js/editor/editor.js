//////////////////////////////////////////////////////
//          2D GAME ENGINE - WEB IDE APP            //
//////////////////////////////////////////////////////



//////////////////////////////
//          Imports         //


// We import the DOM modules' wrapper
import documentInteractionSetup, * as DOM from "./DOMcontrol-wrap.js";

// We import the SQL Control module
import initSQLite, * as SQLITE from "./SQLcontrol.js";



/////////////////////////////////////////
//           Repeatedly Used           //

//
let gameTestWindow = undefined;

//
let assetsToLoad = [];
//
let elementsToLoad = [];

self.setupTestWindow = function(new_window) {
    //
    let allAssets = [];
    //
    SQLITE.forEachInTable(project, "asset", (row) => {
        allAssets.push([row["rowid"], row["type"]]);
    });
    //
    assetsToLoad = allAssets;

    //
    let allElements = [];
    //
    SQLITE.forEachInTable(project, "element", (row) => {
        allElements.push([row["rowid"], row["type"]]);
    });
    //
    elementsToLoad = allElements;

    //
    new_window.self.getMetadataIcon = function() {
        return getBlob(3);
    };
    //
    new_window.self.getMetadataScript = function() {
        return getBlob(2, true);
    };
    //
    new_window.self.getAssetData = function(rowid) {
        return getBlob(SQLITE.getRow(project, "asset", rowid).data);
    };
    //
    new_window.self.getElementScript = function(rowid) {
        return getBlob(SQLITE.getRow(project, "element", rowid).script, true);
    };
    //
    new_window.self.getMetadataConfig = function() {
        return getBlob(1, true);
    };
    //
    new_window.self.getAssetConfig = function(rowid) {
        return getBlob(SQLITE.getRow(project, "asset", rowid).config, true);
    };
    //
    new_window.self.getElementConfig = function(rowid) {
        return getBlob(SQLITE.getRow(project, "element", rowid).config, true);
    };
    //
    new_window.self.getElementID = function(name) {
        let stmt = project.prepare(`SELECT rowid FROM element WHERE name=?;`, [name]);
        stmt.step();
        let res = stmt.get();
        stmt.free();
        return res[0];
    };
    //
    new_window.self.getAssetID = function(name) {
        let stmt = project.prepare(`SELECT rowid FROM asset WHERE name=?;`, [name]);
        stmt.step();
        let res = stmt.get();
        stmt.free();
        return res[0];
    };
    //
    new_window.self.getElementName = function(rowid) {
        const result = SQLITE.getRow(project, "element", rowid).name;
        return ((result === undefined) ? "" : result);
    };
    //
    new_window.self.getAssetName = function(rowid) {
        const result = SQLITE.getRow(project, "asset", rowid).name;
        return ((result === undefined) ? "" : result);
    };
    //
    new_window.self.getElementType = function(rowid) {
        return SQLITE.getRow(project, "element", rowid).type;
    };
    //
    new_window.self.getAssetType = function(rowid) {
        return SQLITE.getRow(project, "asset", rowid).type;
    };
    //
    new_window.self.assetsToLoad = function() {
        let toLoad = assetsToLoad;
        assetsToLoad = [];
        return toLoad;
    };
    //
    new_window.self.elementsToLoad = function() {
        let toLoad = elementsToLoad;
        elementsToLoad = [];
        return toLoad;
    };

    //
    new_window.onbeforeunload = () => {
        //
        if (gameTestWindow !== undefined) {
            //
            gameTestWindow = undefined;
        }
    }

    new_window.onload = (e) => {
        e.target.title = projectName;
        e.target.querySelector("#game-icon").setAttribute("href", document.querySelector("#game-icon-label > img").src);
    };

    gameTestWindow = new_window;
}

//
onpagehide = (e) => {
    //
    if (gameTestWindow !== undefined && !e.persisted) {
        //
        gameTestWindow.close();
    }
};

//
const textEncoder = new TextEncoder();
//
const textDeoder = new TextDecoder();

//
function getBlob(rowid, asText = false) {
    const blob = SQLITE.getRow(project, "blobs", rowid).data;
    return ((asText) ? textDeoder.decode(blob) : blob);
}

// This function loads data from
// the project file into the document
// and overrides previous data in the
// document if there is any.
function loadEditor() {
    //
    if (gameTestWindow !== undefined) {
        //
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

    //
    loadGameIcon(new Blob([SQLITE.getRow(project, "blobs", 3)['data']]));
    //
    document.querySelector("#game-icon-label").onclick = () => {
        document.querySelector("#game-icon-input").value = '';
    };
    //
    document.querySelector("#game-icon-input").oninput = (event) => {
        //
        event.target.files[0].arrayBuffer().then((result) => {
            //
            SQLITE.updateRowValue(project, "blobs", 3,
                'data', new Uint8Array(result));
            //
            loadGameIcon(event.target.files[0]);
        });
    }
}

//
function loadGameIcon(blob) {
    //
    const gameIconImg = document.querySelector("#game-icon-label > img");
    //
    if (gameIconImg.src !== undefined) {
        window.URL.revokeObjectURL(gameIconImg.src);
    }
    //
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
    return null;
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
        // pointer to typesDB.
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
        // pointer to project.
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
            //
            'test-game': () => {
                //
                if (gameTestWindow !== undefined) {
                    //
                    gameTestWindow.close();
                    //
                    gameTestWindow = undefined;
                }
                //
                window.open("/game-test", "GAME TEST");
            },
            //
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


            //
            'config': (table, rowid) => {
                //
                const blobid = (table == 'metadata') ? 1 : SQLITE.getRow(project, table, rowid)['config'];
                //
                const text = textDeoder.decode(SQLITE.getRow(project, "blobs", blobid)['data']);
                //
                return {JSON: JSON.parse(text), blobID: blobid};
            },

            //
            'script': (table, rowid) => {
                //
                let blobid = (table == 'metadata') ? 2 : SQLITE.getRow(project, table, rowid)['script'];
                //
                const text = textDeoder.decode(SQLITE.getRow(project, "blobs", blobid)['data']);
                //
                return {text: text, rowid: blobid};
            }
        }, {



            //////////////////////////////////////////////////
            //          Material Change functions           //


            //
            'config': (configInfo) => {
                //
                const blobid = configInfo.blob;
                //
                const blob = textEncoder.encode(JSON.stringify(configInfo.JSON));
                //
                SQLITE.updateRowValue(project, "blobs", blobid, "data", blob);
                //
                const curTab = document.querySelector("#tabSelected");
                //
                if (curTab.dataset['table'] == "metadata") {
                    return;
                }
                //
                if (curTab.dataset['table'] == "element") {
                    //
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "element", Number(curTab.dataset['tableId']))['type']];
                    //
                    if (elementsToLoad.indexOf(toLoad) == -1) {
                        elementsToLoad.push(toLoad);
                    }
                    return;
                }
                //
                if (curTab.dataset['table'] == "asset") {
                    //
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "asset", Number(curTab.dataset['tableId']))['type']];
                    //
                    if (assetsToLoad.indexOf(toLoad) == -1) {
                        assetsToLoad.push(toLoad);
                    }
                    return;
                }
            },

            //
            'script': (text, blobid) => {
                //
                SQLITE.updateRowValue(project, "blobs", blobid, "data", textEncoder.encode(text));
                //
                const curTab = document.querySelector("#tabSelected");
                //
                if (curTab.dataset['table'] == "metadata") {
                    return;
                }
                //
                if (curTab.dataset['table'] == "element") {
                    //
                    let toLoad = [Number(curTab.dataset['tableId']),
                    SQLITE.getRow(project, "element", Number(curTab.dataset['tableId']))['type']];
                    //
                    if (elementsToLoad.indexOf(toLoad) == -1) {
                        elementsToLoad.push(toLoad);
                    }
                    return;
                }
            }
        }
    );
});