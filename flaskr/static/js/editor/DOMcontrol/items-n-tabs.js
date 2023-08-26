/////////////////////////////////////////////
//          Items and Tabs module          //
/////////////////////////////////////////////

// MDN web docs - Document Object Model:
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model



//////////////////////////////////////
//          Item functions          //


// This function receives a table name,
// a row object, a types map and a optional
// 'open' boolean value (false by default).
// It adds the item which the received row
// represents to its corresponding container
// in the explorer HTML element of the document.
// if the 'open' boolean is true, the item
// will be selected, revealed and ,if it's not
// a folder, opened in a new tab when it's added.
// An item can be a folder, element or asset.
function addItem(table, row, types, open = false) {
    // row['container'] is an ID used
    // to reference the rowid of the
    // container in the folder table.
    // If it equals 0, the container
    // is the base folder(the item-list
    // HTML element in the explorer). if
    // it doesn't equal 0, the container
    // is a real folder, and its class
    // name will be "folder". For styling
    // purposes, the real container HTML element
    // in a real folder is nested like this:
    // folder > .folder-content > ul
    // with "ul" as the real container.
    let container = row['container'];
    container = document.querySelector(`.explorer > .item-list .folder[data-table-id='${container}'], .explorer > .item-list[data-table-id='${container}']`);
    if (container.className == "folder") {
        container = container.querySelector(":scope > .folder-content > ul");
    }

    // We create the HTML element of the item and add it to its container.
    const item = container.appendChild(document.createElement("li"));
    // Its class name will be the table it belongs to (folder/element/asset).
    item.className = table;
    // row['name'] is the name of the item.
    // Its corresponding attribute in the HTML
    // element will be "name".
    item.setAttribute("name", row['name']);
    // row['rowid'] is the row ID of the item
    // in its corresponding table. Its corresponding 
    // attribute in the HTML element will be "data-table-id",
    // which appears in the dataset as dataset['tableId'].
    item.dataset['tableId'] = row['rowid'];

    // This span will make the item's name
    // appear on the document. It needs to be
    // defined differently in a folder item.
    let span;

    // if the item is a folder
    if (table == "folder") {
        // This div will be the switch that makes the folder open and close
        // and will also contain the span which displays the folder's name.
        const fswitch = item.appendChild(document.createElement("div"));
        // The span is defined as the child of the switch HTML element.
        span = fswitch.appendChild(document.createElement("span"));
        // This div is a warpper for the folder's contents.
        const foldercontent = item.appendChild(document.createElement("div"));
        // This div will have the folder's contents inside of it.
        foldercontent.appendChild(document.createElement("ul"));

        // We give class names to some of the HTML elements
        foldercontent.className = "folder-content";
        fswitch.className = "switch";

        // We add the folder state data attribute to the
        // HTML element, and give it the default value "close".
        // This attribute is checked in the style sheet,
        // and will make the folder open or close.
        item.dataset['folderState'] = "close";
        // row['color'] is a hex color string which
        // represents the color of the folder. It's
        // assigned to the folder throught the style
        // attribute, using the "--folder-color" variable
        // which is defined in the style sheet.
        item.setAttribute("style", `--folder-color: ${row['color']};`);
    } else { // if the item isn't a folder, but a element or asset
        // The span is defined as the child of the item HTML element itself.
        span = item.appendChild(document.createElement("span"));
        // row['type'] is an ID used
        // to reference the rowid of the
        // type of this item in the elementType/assetType
        // table from the typesDB. The types map we got
        // as an argument gives us an easyer way to
        // use row['type'] and the table name in order
        // to get the name of the corresponding type.
        // That name will be set to the item's data-type
        // attribute, which is dataset['type'].
        item.dataset['type'] = types.get(table).get(row['type']);

        // a new tab will be opened
        // and selected for this item
        // if 'open' is true. 
        if (open) {
            selectTab(addTab(item));
        }
    }
    
    // We set the span's innerText to
    // row['name']. This will make the
    // span display the item's name.
    span.innerText = row['name'];

    // The new item will be
    // selected and revealed 
    // if 'open' is true.
    if (open) {
        selectItem(item);
        revealItem(item);
    }

    // we return a pointer to the
    // new item in the DOM.
    return item;
}

// This function receives an
// item HTML element and a validation
// function. It tries to remove
// the item from the document, or
// remove multiple items if the
// item is a folder. It returns
// an object which contains a
// messages array with messages
// about items that couldn't be
// removed, and another array with
// a list of more objects which 
// contain the table and rowid
// of every item that was removed.

// The validation function will 
// determine if a certain item is
// removabe or not. It must return
// an appropriate message if the  
// item can't be removed. If it can 
// be removed, the return value
// should be null.
function removeItem(item, onValidation) {

    // We define the arrays
    // we'll use to track removed
    // items and new messages.
    const messages = [];
    const removed = [];

    // If the item is from the folder table
    if (item.className == "folder") {
        // We get all the item inside the folder.
        const items = item.querySelectorAll(":scope > .folder-content > ul > *");
        // We Try to remove the
        // items this item contains
        items.forEach((child) => {
            // We use the same function
            // on this item and save the results.
            const results = removeItem(child, onValidation);
            // We add the new
            // results to our arrays.
            for (let message of results.messages) {
                messages.push(message);
            }
            for (let remove of results.removed) {
                removed.push(remove);
            }
        });

        // If we have no messages, which
        // means that all of our items were removed.
        if (messages.length == 0) {
            // We add this folder's table name
            // and rowid to the removed array.
            removed.push({table: "folder", rowid: item.dataset['tableId']});
            // We remove the folder, because it
            // ended up empty.
            item.remove();
        }
        // We return the result object
        return {messages: messages, removed: removed};
    }

    // We save the table name and
    // rowid of the item for later use.
    const table = item.className;
    const rowid = item.dataset['tableId'];

    // We try to get a message from the
    // validation function to see if we
    // can remove the item or not.
    const validMessage = onValidation(table, rowid, item.dataset['type']);
    // if the message is defined
    if (validMessage !== null) {
        // We add the message to the messages array.
        messages.push(validMessage);
        // We return the result object
        return {messages: messages, removed: removed};
    }
    // We add the item's table name
    // and rowid to the removed array.
    removed.push({table: table, rowid: rowid});
    // We try to get the corresponding tab
    // of the item which we are going to remove.
    const tab = fromItemToTab(item);
    // If it has a corresponding tab
    if (tab !== null) {
        // we remove the tab.
        removeTab(tab);
    }
    // We remove the item.
    item.remove();

    // We return the result object
    return {messages: messages, removed: removed};
}

// This function receives an
// item HTML element and selects it.
// The style sheet makes selected
// items' name appear light green,
// even when the mouse doesn't hover
// over then. The item will also scroll
// into view by default when this
// function is called.
function selectItem(item, scroll = true) {
    // if the item is a folder
    if (item.className == "folder") {
        // the switch should get selected
        item = item.querySelector(":scope > .switch");
    }
    // if the item is already selected
    if (item.id == "itemSelected") {
        // we don't need to select it again,
        // but if we want to scroll into view,
        if (scroll) {
            // we make sure the item is in view.
            item.scrollIntoView();
        }
        return;
    }
    // we get the currently selected item
    const selected = document.getElementById("itemSelected");
    // if it exists
    if (selected !== null) {
        // we deselect it by removing its id attrbute
        selected.removeAttribute("id");
    }
    // we select the item by setting
    // it's id attrbute to "itemSelected"
    item.id = "itemSelected";
    // If we want to scroll into view,
    if (scroll) {
        // we make sure the item is in view.
        item.scrollIntoView();
    }
}

// This function receives an
// item HTML element and opens all
// the folders which contain him,
// and as a result, the item will
// be visable on the item list.
// If the item is already visable,
// this function will have no effect.
function revealItem(item) {
    while (item.parentNode.className != "item-list") {
        // (the item) > ul > .folder-content > (the folder)
        item = item.parentNode.parentNode.parentNode;
        if (item.dataset['folderState'] != "open") {
            item.dataset['folderState'] = "open";
        }
    }
}

// This function receives a folder
// item HTML element and toggles its state
// (from open to close and vice versa)
function toggleFolderState(folder) {
    if (folder.dataset['folderState'] != "close") {
        folder.dataset['folderState'] = "close";
        return;
    }
    folder.dataset['folderState'] = "open";
}



//////////////////////////////////////
//          Tab functions           //


// This function receives an item
// HTML element and creates a tab HTML
// element that corresponds to it in the
// tab-list HTML element of the document.
function addTab(item) {
    // we save the items table
    // and rowid in constants
    // for later use.
    const table = item.className;
    const rowid = item.dataset['tableId'];
    
    // if the item already has a tab
    // or the item is a folder (folders
    // can't have tabs).
    if (document.querySelector(`.editor .tab-section .tab-list .tab[data-table-id='${rowid}'][data-table='${table}']`) != null || table == "folder") {
        // we return null, and we don't create the tab.
        return null;
    }
    
    // We create the tab HTML element and add it to the tab-list HTML element.
    const tab = document.querySelector(".editor .tab-section .tab-list").appendChild(document.createElement("li"));
    // We create a span inside the tab HTML element, which will make the tab's name appear on the document.
    const span = tab.appendChild(document.createElement("span"));
    // We create the x HTML element, which will close the tab when it's clicked.
    const x = document.createElement("button");
    // We add the x HTML element after the span.
    span.after(x);

    // We give class names to some of the HTML elements
    tab.className = "tab";
    x.className = "x";

    // We set the tab's name attribute
    // to the item's name attribute.
    tab.setAttribute("name", item.getAttribute("name"));
    // We set the span's innerText to
    // the tab's name attribute. This 
    // will make the span display the
    // tab's name.
    span.innerText = tab.getAttribute("name");
    // We set the tab's data-table attribute
    // (which is also dataset['table']) to
    // the item's table name.
    tab.dataset['table'] = table;
    // We set the tab's data-table-id attribute
    // (which is also dataset['tableId']) to the item's rowid.
    tab.dataset['tableId'] = rowid;
    // We set the tab's data-type attribute
    // (which is also dataset['type']) to 
    // the item's type name.
    tab.dataset['type'] = item.dataset['type'];

    // we return a pointer to 
    // the new tab in the DOM.
    return tab;
}

// This function receives a
// tab and removes it from the
// tab-list, while trying to
// select another tab if the 
// removed tab was selected.
// The function returns true
// if the selection was changed
// during the removal.
function removeTab(tab) {
    // This constant tells us if
    // the removed tab was the 
    // selected tab.
    const isSelected = tab.id == "tabSelected";
    // We remove the tab.
    tab.remove();
    // If it wasn't the selected tab
    if (!isSelected) {
        // we can return right away
        return false;
    }
    // We'll try to replace the selected 
    // tab with the first tab we can find.
    const replacment = document.querySelector(".editor .tab-section .tab-list .tab");
    // if we found a tab
    if (replacment != null) {
        // We select the tab we
        // found and return right away.
        replacment.id = "tabSelected";
        return true;
    }
    // If we are still here, then there are no tabs left.
    // Therefore, we select tab-filler, which tells the
    // style sheet the tab list is empty.
    document.querySelector(".editor .tab-section .tab-filler").id = "tabSelected";
    return true;
}

// This function receives a tab
// and selects it. The style sheet
// makes selected tabs appear like
// they are closer to the viewer
// than other tabs in the tab list.
// The tab will also scroll into view
// by default when this function is called.
// The function returns true if the 
// selection was changed.
function selectTab(tab, scroll = true) {
    // if the tab is already selected
    if (tab.id == "tabSelected") {
        // we don't need to select it again,
        // but if we want to scroll into view,
        if (scroll) {
            // we make sure the tab is in view.
            tab.scrollIntoView();
        }
        return false;
    }
    // we get the currently selected tab
    const selected = document.getElementById("tabSelected");
    // if it exists
    if (selected !== null) {
        // we deselect it by removing its id attribute.
        selected.removeAttribute("id");
    }
    // we select the tab by setting
    // it's id attrbute to "tabSelected"
    tab.id = "tabSelected";
    // If we want to scroll into view,
    if (scroll) {
        // we make sure the tab is in view.
        tab.scrollIntoView();
    }
    return true;
}



//////////////////////////////////////////
//          Item-Tab functions          //


// This function receives an item
// and tries to return a pointer
// to its corresponding tab.
function fromItemToTab(item) {
    // if the item is a folder
    if (item.className == "folder") {
        // We return null, because we
        // know a folder item can't have
        // a tab.
        return null;
    }
    // We try to use a CSS selector to find the corresponding tab.
    return document.querySelector(`.editor .tab-section .tab-list .tab[data-table-id='${item.dataset['tableId']}'][data-table='${item.className}']`);
    // if null is returned, the tab doesn't exist.
}

// This function receives a tab
// and tries to return a pointer
// to its corresponding item.
function fromTabToItem(tab) {
    // if the tab is from the metadata table
    if (tab.dataset['table'] == "metadata") {
        // We return null, because we
        // know we won't find a metadata
        // item in the item list.
        return null;
    }
     // We try to use a CSS selector to find the corresponding item.
    return document.querySelector(`.explorer > .item-list .${tab.dataset['table']}[data-table-id='${tab.dataset['tableId']}']`);
    // if null is returned, the item doesn't exist.
}

export { addItem, removeItem, selectItem, revealItem, toggleFolderState, addTab, removeTab, selectTab, fromItemToTab, fromTabToItem };