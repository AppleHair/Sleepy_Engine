////////////////////////////////////////////
//          Action Menus module           //
////////////////////////////////////////////

// MDN web docs - Document Object Model:
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
// https://developer.mozilla.org/en-US/docs/Web/API/HTML_DOM_API

// When a action menu is opened,
// the boolean which corresponeds
// with its type will be set to true.
// When the closeActionMenus function
// runs, it checks if at least one of 
// these booleans are true, and if that
// is true, it sets both of them false
// and closes the opened action menu.
let inContextMenu = false;
let inDropDownMenu = false;

// This is a reference to the context menu HTML element
const contextMenu = document.querySelector("body > .action-menu");
// This is a node list of every option 
// the context menu provides for any item
const contextMenuOptions = contextMenu.querySelectorAll(":scope > li");
// This object will have corresponding
// functions to every context menu option.
// It's defined in the contextMenuSetup function.
let contextMenuFn = {};

//
function actionMenusSetup(dropDownFn = {}, contextFn = {}) {
    // we give the dropDownMenuSetup function
    // and contextMenuSetup function their
    // option functions and run them
    dropDownMenuSetup(dropDownFn);
    contextMenuSetup(contextFn);
}

// This function sets up the context menu's
// behavior. It's called by documentInteractionSetup,
// which gives it its option functions.
function contextMenuSetup(optionsFn = {}) {
    // We save the option functions
    // in the contextMenuFn variable.
    contextMenuFn = optionsFn;
}

// This function receives an 
// item/item-list HTML element and
// x and y coordinates on the
// page. It opens a context menu 
// on the received x and y positions 
// on the page, and shows options
// related to the received item.
function openContextMenu(item, x, y) {
    // We set the left and top positioning
    // propertys to the x and y coordinates.
    contextMenu.style.left = `${x}px`;
    contextMenu.style.top = `${y}px`;

    // We itrate on the context menu options
    contextMenuOptions.forEach((option) => {
        // if the option fits the item received
        if (option.dataset['table'].includes(item.className)) {
            // we reveal the option
            option.style.display = "block";
            // we get this option's function
            const optionFunc = contextMenuFn[option.getAttribute("name")];
            // if this function is defined
            if (optionFunc !== undefined) {
                // We make the function run
                // when the option is clicked.
                option.onclick = () => optionFunc(item);
            }
            return;
        }
        // We hide the option
        option.style.display = "none";
    });
    // We close other action menus
    closeActionMenus();
    // We set inContextMenu to true,
    // and tell other events that 
    // the context menu is open.
    inContextMenu = true;
    // We reveal the context menu by
    // setting its id to "menuSelected"
    contextMenu.id = "menuSelected";
}

// This function closes all 
// action menus if any are open.
function closeActionMenus() {
    // if any action menu is open
    if (inDropDownMenu || inContextMenu) {
        // close the currently open action
        // menu by removing its id.
        document.getElementById("menuSelected").removeAttribute("id");
        // We set both booleans to false
        // because all of the action menus
        // are now closed.
        inContextMenu = false; inDropDownMenu = false;
    }
}

// This function sets up the
// drop down menus. It's called by
// documentInteractionSetup, which
// gives it its option functions.
function dropDownMenuSetup(optionsFn = {}) {
    // We get a reference to the menu bar HTML element
    const menubar = document.querySelector(".menu-bar");

    // This function will make
    // the menu buttons toggle
    // the drop down menus'
    // visibility when they're
    // clicked.
    menubar.addEventListener("click", (event) => {
        //
        if (event.target.matches(".project-menu .menu-button, .project-menu .menu-button *")) {
            //
            const button = event.target.closest(".menu-button");
            // if a drop down menu
            // is already open.
            if (inDropDownMenu) {
                // We return and
                // let the document
                // close the menu
                return;
            }
            // We make sure the menu won't get
            // closed by the page wrapper
            event.stopPropagation();
            // We close other action menus
            closeActionMenus();
            // We set inDropDownMenu to true,
            // and tell other events that 
            // a drop down menu is open.
            inDropDownMenu = true;
            // We reveal the drop down menu by
            // setting the id of it's button to 
            // "menuSelected"
            button.id = "menuSelected";
        }
    });
    // This function will make
    // the drop down menus switch
    // between each other when you
    // hover over their buttons, if
    // any of them are currently selected.
    menubar.addEventListener("mouseover", (event) => {
        //
        if (event.target.matches(".project-menu .menu-button, .project-menu .menu-button *")) {
            //
            const button = event.target.closest(".menu-button");
            // if a drop down menu is open.
            if (inDropDownMenu) {
                // close it by removing its id
                document.getElementById("menuSelected").removeAttribute("id");
                // reveal this drop down menu by
                // setting the id of it's button 
                // to "menuSelected"
                button.id = "menuSelected";
            }
        }
    });
    // We itrate on each action menu's options
    menubar.querySelectorAll(".action-menu > li").forEach((option) => {
        // we get this option's function
        const optionFunc = optionsFn[option.getAttribute("name")];
        // if this function is defined
        if (optionFunc !== undefined) {
            // We make the function run
            // when the option is clicked.
            option.onclick = optionFunc;
        }
    });
}

export { closeActionMenus, openContextMenu }
export default actionMenusSetup;