//////////////////////////////////////////////////
//          DOM Control Wrapper Module          //
//////////////////////////////////////////////////

// This module and all the others use the HTML DOM
// to access, change and control the behavior of
// HTML elements in the document. The comments in
// every module describe the way various methods 
// and classes from the DOM have been used, but 
// descriptions of the methods themselves will 
// not be provided, because there are too meny.


// MDN web docs - Document Object Model (if you really need it):
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model



//////////////////////////////
//          Imports         //


import materialSetup, {switchMaterial, 
    resetMaterial} from "./DOMcontrol/material.js";

import actionMenusSetup, {openContextMenu,
    closeActionMenus} from "./DOMcontrol/action-menus.js";

import {openMessageWindow, 
    openInputWindow} from "./DOMcontrol/pop-up-window.js";

import { addItem, removeItem, selectItem, 
    revealItem, toggleFolderState, addTab, 
    removeTab, selectTab, fromItemToTab, 
    fromTabToItem } from "./DOMcontrol/items-n-tabs.js";

//////////////////////////////////////////////////
//          Document Interaction Setup          //


// This function's purpose is to set up
// pre-defined HTML elements to be
// interactive and call different functions
// according to the users activity on the page.
// It receives four objects: the first contains
// callbacks for the drop down menus, the second
// contains callbacks for the context menu, the 
// third contains callbacks for when the material
// mode is switched, aka the user switches from
// one editor tab to another, and the fourth contains
// callbacks for whenthe contents of the displayed
// material are changed, aka the user edits the game.
function documentInteractionSetup(dropDownFn = {}, contextFn = {}, switchFn = {}, changeFn = {}) {

    //
    document.addEventListener("click", (event) => {
        //
        closeActionMenus();
        // This function makes the item HTML element get selected when it's
        // clicked, and also selects its corresponding tab if it exists.
        if (event.target.matches(".item-list *, .metadata, .metadata *")) {
            //
            const item = event.target.closest(".element, .asset, .folder, .metadata");
            //
            selectItem(item, false);
            // This function makes the switch HTML element open and close the folder
            // when it's clicked, and also makes the folder get selected.
            if (item.className == "folder") {
                toggleFolderState(item);
                return;
            }
            //
            const tab = fromItemToTab(item);
            //
            if (tab !== null) {
                selectTab(tab);
            }
        }
    });

    // 
    document.addEventListener("contextmenu", (event) => {
        //
        closeActionMenus();
        //
        if (event.target.matches(".item-list, .item-list *")) {
            //
            const item = event.target.closest(".element, .asset, .folder, .item-list");
            //
            openContextMenu(item,event.pageX, event.pageY);
            // We select the item only if
            // it's actually an item, and
            // not the item-list.
            if (item.className != "item-list") {
                // we don't want to scroll
                // into view because that 
                // can make the item move
                selectItem(item, false);
            }
        }
        //
        event.preventDefault();
    });

    // This function makes a new tab open for an item when that
    // item is double clicked, and also selects the new tab.
    document.querySelector(".explorer").addEventListener("dblclick", (event) => {
        //
        if (event.target.matches(".element, .element *, .asset, .asset *, .metadata, .metadata *")) {
            //
            const item = event.target.closest(".element, .asset, .metadata");
            //
            const tab = addTab(item);
            //
            if (tab !== null) {
                //
                const changed = selectTab(tab);
                //
                if (changed) {
                    switchMaterial();
                }
            }
        }
    });

    // This function selects the tab 
    // and selects and reveals the tab's 
    // corresponding item if this tab's
    // span is clicked.
    document.querySelector(".tab-list").addEventListener("click", (event) => {
        //
        if (event.target.matches(".tab, .tab *")) {
            //
            let changed;
            //
            const tab = event.target.closest(".tab");
            // This function removes the tab
            // if his x HTML element is clicked.
            if (event.target.closest(".x") !== null) {
                //
                changed = removeTab(tab);
                //
                if (changed) {
                    switchMaterial();
                }
                return;
            }
            //
            changed = selectTab(tab, false);
            //
            if (changed) {
                switchMaterial();
            }
            //
            const item = fromTabToItem(tab);
            //
            if (item !== null) {
                selectItem(item);
                revealItem(item);
            }
        }
    });

    // We create a new ResizeObserver
    // object and define its behavior
    const resizeObserver = new ResizeObserver((entries) => {
        // We itrate on the observer's entries
        for (let entry of entries) {
            // if the entry's target is the explorer
            if (entry.target.className == "explorer") {
                // we set the --explorer-width css variable
                // to the explorer's new width in order to
                // use css to give the editor section a 
                // defined width, and make the tab section
                // overflow instead of stretch the page when
                // it gets too wide
                document.querySelector(".editor").style = `--explorer-width: ${entry.contentBoxSize[0].inlineSize}px;`;
                // we continue the itration
                continue;
            }
            // if the target is not the explorer,
            // than it should be the header, which
            // should be the only other HTML element observed
            // by this object. we set the --header-height
            // css variable to the header's new width 
            // in order to use css to give the workspace
            // a defined height, and make the item list 
            // and the material section's contents
            // overflow instead of stretch the page
            // when they get too tall
            document.querySelector(".workspace").style = `--header-height: ${entry.contentBoxSize[0].blockSize}px;`;
        }
    }); 
    // We make resizeObserver observe the explorer
    resizeObserver.observe(document.querySelector(".explorer"));
    // We make resizeObserver observe the header
    resizeObserver.observe(document.querySelector(".page-wrapper > header"));

    // we make the tab section close the action menus
    // and scroll when the mouse wheel rolls on it.
    const tabsection = document.querySelector(".tab-section");
    tabsection.onwheel = (event) => {
        closeActionMenus();
        tabsection.scrollBy({left: (event.deltaY / 4), behavior: "instant" });
    };

    // we make the explorer close the action menus
    // and scroll when the mouse wheel rolls on it.
    const explorer = document.querySelector(".explorer");
    explorer.onwheel = (event) => {
        closeActionMenus();
        explorer.scrollBy({top: (event.deltaY / 4), behavior: "instant" });
    };

    // we make the header close the action menus and
    // scroll the document horizontally when the mouse wheel rolls on it.
    document.querySelector(".page-wrapper > header").onwheel = (event) => {
        closeActionMenus();
        document.querySelector(":scope").scrollBy({left: (event.deltaY / 4), behavior: "instant" });
    };


    //
    actionMenusSetup(dropDownFn, contextFn);

    //
    materialSetup(switchFn, changeFn);
}


// We export the things we want the
// importing script to be able to use.

export { addItem, removeItem, openMessageWindow, openInputWindow, switchMaterial, resetMaterial };
export default documentInteractionSetup;