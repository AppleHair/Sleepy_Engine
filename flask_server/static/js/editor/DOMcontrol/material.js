///////////////////////////////////////
//          Material Module          //
///////////////////////////////////////

// MDN web docs - Document Object Model:
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
// https://developer.mozilla.org/en-US/docs/Web/API/HTML_DOM_API

// This module is responsible for providing
// a codemirror mode for the rhai scripting language
// (used in rhai playground demo: https://rhai.rs/playground/stable/)
import { wasm, wasmLoadPromise } from "../../libs/codemirror5/mode/rhai-playground/wasm_loader.js";

// This is a reference to the material section HTML element
const materialsection = document.querySelector(".editor > .material-section");

// These are callbacks which get called when
// the user interacts with the material
let onSwitchOf = {
    'config': null,
    'script': null,
    'preview': null
}
let onChangeTo = {
    'config': null,
    'script': null
}
let beforeChange = {
    'config-input': null,
    'config-minus': null,
    'config-plus': null
}

// Determines which components are
// available for each material mode
const modeComponents = {
    'sprite': ['config', 'preview'],
    'audio': ['preview'],
    'font': ['preview'],
    'object': ['config', 'preview', 'script'],
    'scene': ['config', 'preview', 'script'],
    'state': ['config', 'script'],
    'none': ['preview'],
}

// These are references to the
// buttons which switch between
// the material's script and
// config components
const toConfigButton = document.querySelector("#toConfig-button");
const toScriptButton = document.querySelector("#toScript-button");

// This function sets up the functionality
// of the material section. It receives
// three objects: the first contains callbacks
// for when the material mode is switched, aka
// the user switches from one editor tab to another,
// the second contains callbacks for when the contents
// of the displayed material are changed, aka the user
// edits the game, and the third contains callbacks
// for before the material get changed.
function materialSetup(switchFn = {}, changeFn = {}, beforeChangeFn = {}) {
    // receive the callbacks
    onSwitchOf = switchFn;
    onChangeTo = changeFn;
    beforeChange = beforeChangeFn;
    // set up the config and script components
    configSetup();
    scriptSetup();
    // make the buttons switch between the
    // material's script and config components
    toConfigButton.onclick = () => {
        const curTab = document.querySelector("#tabSelected");
        curTab.removeAttribute("data-in-script");
        switchMaterial();
    }
    toScriptButton.onclick = () => {
        const curTab = document.querySelector("#tabSelected");
        curTab.setAttribute("data-in-script", '');
        switchMaterial();
    }
}

// This function switches the material's
// mode according to the currently selected
// tab. It also switches between the material's
// script and config components if the tab
// is in script mode.
function switchMaterial() {
    // get the currently selected tab
    const curTab = document.querySelector("#tabSelected");
    // get the material's current mode
    // which will be switched from and
    // become the previous mode
    const prvMode = materialsection.dataset['mode'];
    switchMode();
    // if the previous mode was script
    if (prvMode == 'script') {
        // clear the script component
        scriptBlob = 0;
        rhaiScript.swapDoc(codemirrorDocMap.get(scriptBlob));
        rhaiScript.refresh();
        // switch the material's mode
        // according to the current tab
        switchMode(curTab.hasAttribute("data-in-script") ? "script" : curTab.dataset['type'], 
        curTab.dataset['table'], curTab.dataset['tableId']);
        rhaiScript.refresh();
        return;
    }
    // if the previous mode used the config component
    if (modeComponents[prvMode].includes('config')) {
        // clear the config component
        clearConfig(prvMode);
        configInfo.JSON = {};
        configInfo.form = '';
        configInfo.blob = 0;
    }
    // switch the material's mode
    // according to the current tab
    switchMode(curTab.hasAttribute("data-in-script") ? "script" : curTab.dataset['type'], 
        curTab.dataset['table'], curTab.dataset['tableId']);
}

// This function switches the material's
// mode. It receives three arguments: the
// first is the mode to switch to, the second
// and third are the table and rowid of the
// item associated with the material. 
function switchMode(mode, table, rowid) {
    // if the mode is undefined,
    // it's set to 'none'
    if (mode === undefined) {
        mode = 'none';
    }
    // if the mode is being switched
    // to script, and the table and
    // rowid are defined, we make the
    // element's script display in the
    // material section.
    if (mode == 'script' && table !== undefined && rowid !== undefined) {
        const scriptInfo = onSwitchOf['script'](table, rowid);
        scriptBlob = scriptInfo.rowid;
        if (codemirrorDocMap.get(scriptBlob) === undefined) {
            codemirrorDocMap.set(0,rhaiScript.getDoc().copy(true));
            rhaiScript.setValue(scriptInfo.text);
            rhaiScript.clearHistory();
            codemirrorDocMap.set(scriptBlob, rhaiScript.getDoc());
        } else {
            rhaiScript.swapDoc(codemirrorDocMap.get(scriptBlob));
            if (rhaiScript.getValue() != scriptInfo.text) {
                rhaiScript.setValue(scriptInfo.text);
            }
        }
        // set the material's mode to script
        materialsection.dataset['mode'] = mode;
        rhaiScript.refresh();
        return;
    }
    // if the mode is being switched
    // to a mode that uses the config
    // component, and the table and
    // rowid are defined, we make the
    // item's config display in the
    // material section.
    if (modeComponents[mode].includes('config') && table !== undefined && rowid !== undefined) {
        const configBlobInfo = onSwitchOf['config'](table, rowid);
        configInfo.JSON = configBlobInfo.JSON;
        configInfo.form = mode;
        configInfo.blob = configBlobInfo.blobID;
        loadConfig(mode);
    }
    // set the material's mode
    materialsection.dataset['mode'] = mode;
}

// This function resets the material
// according to its mode. It receives
// the mode as an argument, and switches
// to the 'none' mode.
function resetMaterial(mode) {
    // set the material's mode to none
    materialsection.dataset['mode'] = 'none';
    // clear the config component
    clearConfig(mode);
    configInfo.JSON = {};
    configInfo.form = '';
    configInfo.blob = 0;
    // clear the script component
    if (mode == 'script') {
        scriptBlob = 0;
        rhaiScript.swapDoc(codemirrorDocMap.get(scriptBlob));
    }
    rhaiScript.refresh();
    // clear the codemirror document map
    codemirrorDocMap.clear();
}



//////////////////////////////////////////////
//          Configuration handling          //


// These are references to the
// different config component's
// forms which are used for
// different kinds of items.
const configForms = {
    'sprite': document.querySelector("#sprite-config"),
    'object': document.querySelector("#object-config"),
    'scene': document.querySelector("#scene-config"),
    'state': document.querySelector("#state-config"),
};

// This is an object which contains
// information about the config component
// and the content it should displays and
// keep track of.
let configInfo = {
    JSON: {},
    form: '',
    blob: 0
};

// This function helps you get to a path
// in the JSON object in which an input
// HTML element's value should be stored.
// You can use it to easily reference a
// value in the JSON object, using its
// associated input HTML element.
function getJSONScope(htmlElement) {
    // find the path to the input HTML element's
    // corresponding variable in the JSON object
    const path = [];
    while (htmlElement.className != "config") {
        htmlElement = htmlElement.parentNode;
        if (htmlElement.hasAttribute("name")) {
            path.push(htmlElement.getAttribute("name"));
        }
    }
    // reverse the path
    path.reverse();
    // use the path array to obtain
    // a direct reference to it in
    // the JSON object
    let container = configInfo.JSON;
    for (let key of path) {
        container = container[key];
    }
    // return the reference
    return container;
}

// This function sets up the functionality
// of the material's config component.
function configSetup() {
    
    materialsection.addEventListener("input", (event) => {
        // if the input is not a config input, ignore it.
        if (event.target.matches(".li-template input, #game-icon-input, #rhai-script *")) {
            return;
        }
        // get the input HTML element
        const input = event.target;
        // filter the value of the input
        const value = beforeChange['config-input'](input, getJSONScope(input)[input.getAttribute("name")]);
        // reset the input's value
        input.value = value;
        // update the JSON object
        getJSONScope(input)[input.getAttribute("name")] = 
            (input.type == "number" || input.type == "range") ? Number(input.value) : input.value;
        // update the project file
        onChangeTo['config'](configInfo);
    });

    materialsection.addEventListener("click", (event) => {
        // if the clicked element is not a plus or minus button, ignore it.
        if (event.target.matches(":not(.plus-button, .minus-button)")) {
            return;
        }
        // get the button HTML element
        const button = event.target;
        // if the button is a plus button
        // and there's nothing preventing
        // the addition of a new item
        if (button.className == "plus-button" && beforeChange['config-plus'](button.parentNode)) {
            // add an item to the JSON array
            const item = addItemToConfigArray(button.parentNode, getJSONScope(button).length);
        // if the button is a minus button
        // and there's nothing preventing
        // the deletion of a new item
        } else if (button.className == "minus-button" && beforeChange['config-minus'](button.parentNode)) {
            // get the li of the member
            const li = button.parentNode;
            // get the li's index
            const liname = li.querySelector(":scope [name]").getAttribute("name");
            // get the JSON array HTML element itself
            const jsonArray = li.parentNode;
            // itrate on the array's members
            jsonArray.querySelectorAll(":scope > li").forEach((item) => {
                // get the member's input HTML element
                item = item.querySelector(":scope [name]");
                // if the member's index is smaller than the li's index
                if (parseInt(item.getAttribute("name")) <= parseInt(liname) ) {
                    return;
                }
                // decrease the member's index by one
                item.setAttribute("name", parseInt(item.getAttribute("name")) - 1);
            });
            // if the array is a layers array
            // do the same for all the layer-user inputs
            if (jsonArray.getAttribute("name") == "layers") {
                document.querySelectorAll('.layer-user:not(.li-template *)').forEach((user) => {
                    if (parseInt(user.value) <= parseInt(liname)) {
                        return;
                    }
                    user.value = parseInt(user.value) - 1;
                    getJSONScope(user)[user.getAttribute("name")] = user.value;
                });
            }
            // remove the member from the JSON array
            getJSONScope(li).splice(parseInt(liname), 1);
            // remove the li from the HTML
            li.remove();
        }
        // update the project file
        onChangeTo['config'](configInfo);
    });
}

// This function clears one of the config
// component's forms according to the received mode.
function clearConfig(mode) {
    // get the form
    const form = configForms[mode];
    // if the form is undefined, ignore it.
    if (form === undefined) {
        return;
    }
    // remove all the li's from the form
    // not including the version list from
    // the state manager's config form
    form.querySelectorAll(".json-array:not([name=\"version\"]) > li").forEach((li) => {li.remove();});
    // clear all the input fields,
    // not including the game icon input
    // and template li elements
    form.querySelectorAll(".json-field:not(#game-icon) > input").forEach((input) => {
        if (input.matches(".li-template input")) { return; }
        input.removeAttribute("value");
    });
}

// This function loads one of the config
// component's forms according to the received mode.
function loadConfig(mode) {
    // get the form
    const form = configForms[mode];
    // if the form is undefined, ignore it.
    if (form === undefined) {
        return;
    }

    // This function determines which
    // loadVariable method to use for a
    // specific HTML element.
    function determineLoadMethod(htmlElement, scope) {
        if (htmlElement.className.includes('json-field')) {
            loadVariable(htmlElement.querySelector(":scope > input"), scope, "field");
        }
        if (htmlElement.className.includes('json-object')) {
            loadVariable(htmlElement, scope, "object");
        }
        if (htmlElement.className.includes('json-array')) {
            loadVariable(htmlElement, scope, "array");
        }
    }
    // This function loads a variable from the
    // JSON object according to the received
    // HTML element, method and scope from
    // the JSON object itself.
    function loadVariable(htmlElement, scope, method) {
        // get the variable's key
        let key = htmlElement.getAttribute("name");
        // if the key is null, ignore it.
        if (key === null) {
            return;
        }

        // if the method is field, load the
        // variable's value into the input field
        if (method == "field") {
            htmlElement.value = scope[key];
            return;
        }
        // if the method is object, itrate
        // on the object's members and load
        // them into the HTML element
        if (method == "object") {
            htmlElement.querySelectorAll(":scope > *").forEach((attr) => {
                determineLoadMethod(attr, scope[key]);
            });
        }
        // if the method is array, itrate
        // on the array's members and load
        // them into the HTML element
        if (method == "array") {
            let li;
            const isVersion = htmlElement.getAttribute("name") == "version";
            // itrate on the array's members
            for (let i = 0 ; i < scope[key].length ; i++) {
                // if the array is the version array
                if (isVersion) {
                    // get the corresponding li
                    // to the current index
                    // anf load the variable into it
                    li = htmlElement.querySelector(`input[name="${i}"]`);
                    loadVariable(li, scope[key], "field");
                    continue;
                }
                // add an item to the array
                // using the current index
                li = addItemToConfigArray(htmlElement, i, false);
                // load the variable into the item
                determineLoadMethod(li, scope[key]);
            }
        }
    }
    // Get the JSON object
    const JSON = configInfo.JSON;
    // itrate on the form's deapest
    // children and load the variables
    // into them.
    form.querySelectorAll(":scope > *:not(#game-icon)").forEach((htmlElement) => {
        determineLoadMethod(htmlElement, JSON);
    });
}

// This function adds an item to a JSON array.
// It receives the array's HTML element, the index
// of the item, and a boolean which determines
// whether to update the JSON object according
// to the new item.
function addItemToConfigArray(jsonArray, index, update = true) {
    // This function determines which
    // addVariable method to use for a
    // specific HTML element.
    function determineAddMethod(htmlElement, scope) {
        if (htmlElement.className.includes('json-field')) {
            addVariable(htmlElement.querySelector(":scope > input"), scope, "field");
        }
        if (htmlElement.className.includes('json-object')) {
            addVariable(htmlElement, scope, "object");
        }
        if (htmlElement.className.includes('json-array')) {
            addVariable(htmlElement, scope, "array");
        }
    }
    // This function adds a variable to the
    // JSON object according to the received
    // HTML element, method and scope from
    // the JSON object itself.
    function addVariable(htmlElement, scope, method) {
        // get the variable's key
        let key = htmlElement.getAttribute("name");
        // if the key is null, ignore it.
        if (key === null) {
            return;
        }

        // if the method is field, add the
        // variable's value into the input field
        if (method == "field") {
            // filter the value of the input
            const value = beforeChange['config-input'](htmlElement, -1);
            // reset the input's value
            htmlElement.value = value;
            // update the JSON object
            scope[key] = (htmlElement.type == "number" || htmlElement.type == "range") ? Number(htmlElement.value) : htmlElement.value;
        }
        // if the method is object, itrate
        // on the object's members and add
        // them into a new object in the
        // JSON object
        if (method == "object") {
            scope[key] = {};
            htmlElement.querySelectorAll(":scope > *").forEach((attr) => {
                determineAddMethod(attr, scope[key]);
            });
        }
        // if the method is array, itrate
        // on the array's members and add
        // them into a new array in the
        // JSON object
        if (method == "array") {
            scope[key] = [];
            htmlElement.querySelectorAll(":scope > li > :is(json-field, json-object, json-array)").forEach((item) => {
                determineAddMethod(item, scope[key]);
            });
        }
    }

    // create a new li
    const li = document.createElement("li");
    // set the li's contents according to the template
    li.innerHTML = jsonArray.querySelector(":scope > .li-template").innerHTML;
    // add the li to the array
    // before the plus button
    // (at the end of the array)
    jsonArray.querySelector(":scope > .plus-button").before(li);
    // get the li's contents
    const licontent = li.querySelector(":scope > *");

    // if the li's contents is a json-field,
    if (licontent.className.includes('json-field')) {
        // get its input HTML element
        const input = licontent.querySelector(":scope > input");
        // set the input's name to the index
        input.setAttribute("name", index);

        // update the JSON object
        // if the update argument is true
        if (update) {
            addVariable(input, getJSONScope(input), "field");
        }
        // return the li's contents
        return licontent;
    }
    // set the li's contents name to the index
    licontent.setAttribute("name", index);

    // update the JSON object
    // if the update argument is true
    if (update) {
        determineAddMethod(licontent, getJSONScope(licontent));
    }
    // return the li's contents
    return licontent;
}

///////////////////////////////////////
//          Script handling          //

// This is a reference to the form
// in which the script editor is placed
const scriptForm = document.getElementById("rhai-script");

// This is the blob id of the currently
// displayed script
let scriptBlob = 0;

// This is the codemirror editor
// which displays the rhai scripts
const rhaiScript = CodeMirror(scriptForm, {
    value: "",
    mode: null,
    theme: "dracula",
    tabSize: 2,
    indentUnit: 2,
	indentWithTabs: true,
	smartIndent: true,
	lineNumbers: true,
	matchBrackets: true,
    highlightSelectionMatches: true,
    autoCloseBrackets: {
        pairs: `()[]{}''""`,
        closeBefore: `)]}'":;,`,
        triples: "",
        explode: "()[]{}",
    },
    scrollbarStyle: "native"
})

// This is a map which keeps track
// of the edits the user makes to
// the different scripts using
// different codemirror documents.
const codemirrorDocMap = new Map();

// This function sets up the functionality
// of the material's script component
// (codemirror editor).
function scriptSetup() {
    rhaiScript.on("change", () => {
        // Update the project file with
        // the changes to the script
        onChangeTo['script'](rhaiScript.getValue(), scriptBlob);
        codemirrorDocMap.set(scriptBlob, rhaiScript.getDoc());
    });
}

// initialize the rhai playground wasm module
wasmLoadPromise.then(() => {
    // Give the wasm module the ability to
    // parse codemirror's content
    wasm.init_codemirror_pass(CodeMirror.Pass);

    // Define the rhai codemirror mode
    // while giving the constructor
    // the indentUnit value it needs
    CodeMirror.defineMode("rhai", (cfg) => {
        return new wasm.RhaiMode(cfg.indentUnit);
    });

    // Set the codemirror editor's
    // mode to the rhai mode
    rhaiScript.setOption("mode", "rhai");
});

// We export the things we want the
// importing script to be able to use.

export { switchMaterial, resetMaterial };
export default materialSetup;
