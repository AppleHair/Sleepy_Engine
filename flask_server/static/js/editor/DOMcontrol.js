//////////////////////////////////////////
//          DOM Control Module          //
//////////////////////////////////////////

// This module and all the others use the HTML DOM
// to access, change and control the behavior of
// HTML elements in the document. The comments in
// every module describe the way various methods 
// and classes from the DOM have been used, but 
// descriptions of the methods themselves will 
// not be provided, because there are too meny.

// MDN web docs - Document Object Model (if you really need it):
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
// https://developer.mozilla.org/en-US/docs/Web/API/HTML_DOM_API

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
// callbacks for when the contents of the displayed
// material are changed, aka the user edits the game.
function documentInteractionSetup(dropDownFn = {}, contextFn = {}, switchFn = {}, changeFn = {}, beforeChangeFn = {}) {
    document.addEventListener("click", (event) => {
        // any actions menu will be closed
        // when the document is clicked
        closeActionMenus();
        // if an item was clicked on
        if (event.target.matches(".item-list *, .metadata, .metadata *")) {
            // find the item that was clicked on
            const item = event.target.closest(".element, .asset, .folder, .metadata");
            // select the item
            selectItem(item, false);
            // if the item is a folder, make it
            // switch between its open and closed states
            if (item.className == "folder") {
                toggleFolderState(item);
                return;
            }
            // find the item's corresponding tab
            const tab = fromItemToTab(item);
            // select the tab if it exists
            if (tab !== null) {
                selectTab(tab);
                switchMaterial();
            }
        }
    });
    document.addEventListener("contextmenu", (event) => {
        // any actions menu will be closed
        // when the document is clicked
        closeActionMenus();
        // if an item was right clicked on
        if (event.target.matches(".item-list, .item-list *")) {
            // find the item that was right clicked on
            const item = event.target.closest(".element, .asset, .folder, .item-list");
            // open the context menu for the item
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
        // prevent the default
        // context menu from appearing
        event.preventDefault();
    });
    document.querySelector(".explorer").addEventListener("dblclick", (event) => {
        // if an item was double clicked on
        if (event.target.matches(".element, .element *, .asset, .asset *, .metadata, .metadata *")) {
            // find the item that was double clicked on
            const item = event.target.closest(".element, .asset, .metadata");
            // create a new tab for the item
            const tab = addTab(item);
            // if the item already has a tab
            if (tab !== null) {
                // select the tab
                const changed = selectTab(tab);
                // switch the material accordingly
                if (changed) {
                    switchMaterial();
                }
            }
        }
    });
    document.querySelector(".tab-list").addEventListener("click", (event) => {
        // if a tab was clicked on
        if (event.target.matches(".tab, .tab *")) {
            let changed;
            // find the tab that was clicked on
            const tab = event.target.closest(".tab");
            // if the tab's close button was clicked on
            if (event.target.closest(".x") !== null) {
                // remove the tab
                changed = removeTab(tab);
                // switch the material accordingly
                if (changed) {
                    switchMaterial();
                }
                return;
            }
            // if the tab's span was
            // clicked on, select the tab
            changed = selectTab(tab, false);
            // switch the material accordingly
            if (changed) {
                switchMaterial();
            }
            // find the tab's corresponding item
            const item = fromTabToItem(tab);
            // select and reveal the item if it exists
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
    // give the action menus their callback functions
    actionMenusSetup(dropDownFn, contextFn);
    // give the material section its callback functions
    materialSetup(switchFn, changeFn, beforeChangeFn);
}

// We export the things we want the
// importing script to be able to use.

export { addItem, removeItem, fromItemToTab, openMessageWindow, openInputWindow, switchMaterial, resetMaterial };
export default documentInteractionSetup;