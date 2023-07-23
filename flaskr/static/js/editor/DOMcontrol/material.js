///////////////////////////////////////
//          Material Module          //
///////////////////////////////////////

// MDN web docs - Document Object Model:
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model


//
const materialsection = document.querySelector(".editor > .material-section");

//
let onSwitchOf = {
    'config': null,
    'script': null,
    'preview': null
}

//
let onChangeTo = {
    'config': null,
    'script': null
}

//
const modeComponents = {
    'sprite': ['config', 'preview'],
    'audio': ['preview'],
    'font': ['preview'],
    'object': ['config', 'preview', 'script'],
    'scene': ['config', 'preview', 'script'],
    'game': ['config', 'script'],
    'project': ['config'],
    'loading': ['preview'],
    'none': ['preview'],
}

//
const toConfigButton = document.querySelector("#toConfig-button");
//
const toScriptButton = document.querySelector("#toScript-button");

//
function materialSetup(switchFn = {}, changeFn = {}) {
    //
    onSwitchOf = switchFn;
    onChangeTo = changeFn;
    //
    configSetup();
    scriptSetup();
    //
    toConfigButton.onclick = () => {
        //
        const curTab = document.querySelector("#tabSelected");
        //
        curTab.removeAttribute("data-in-script");
        //
        switchMaterial();
    }
    toScriptButton.onclick = () => {
        //
        const curTab = document.querySelector("#tabSelected");
        //
        curTab.setAttribute("data-in-script", '');
        //
        switchMaterial();
    }
}

//
function switchMaterial() {
    //
    const curTab = document.querySelector("#tabSelected");
    //
    const prvMode = materialsection.dataset['mode'];
    //
    switchMode('loading');
    //
    if (prvMode == 'script') {
        //
        scriptBlob = 0;
        luaScript.setValue('');
        //
        luaScript.refresh();
        //
        switchMode(curTab.hasAttribute("data-in-script") ? "script" : curTab.dataset['type'], 
        curTab.dataset['table'], curTab.dataset['tableId']);
        //
        luaScript.refresh();
        return;
    }
    //
    for (let comp of modeComponents[prvMode]) {
        switch(comp) {
            //
            case 'config':
                //
                clearConfig(prvMode);
                //
                configInfo.JSON = {};
                configInfo.form = '';
                configInfo.blob = 0;
                break;
            //
            case 'preview':
                break;
            //
            case 'script':
                break;
            default:

        }
    }
    //
    switchMode(curTab.hasAttribute("data-in-script") ? "script" : curTab.dataset['type'], 
        curTab.dataset['table'], curTab.dataset['tableId']);
}

//
function switchMode(mode, table, rowid) {
    //
    if (mode === undefined) {
        mode = 'none';
    }
    //
    if (mode == 'script' && table !== undefined && rowid !== undefined) {
        //
        const scriptInfo = onSwitchOf['script'](table, rowid);
        luaScript.setValue(scriptInfo.text);
        scriptBlob = scriptInfo.rowid;
        //
        materialsection.dataset['mode'] = mode;
        //
        luaScript.refresh();
        return;
    }
    //
    for (let comp of modeComponents[mode]) {
        switch(comp) {
            case 'config':
                if (table === undefined || rowid === undefined) {
                    break;
                }
                const configBlobInfo = onSwitchOf['config'](table, rowid);
                configInfo.JSON = configBlobInfo.JSON;
                configInfo.form = mode;
                configInfo.blob = configBlobInfo.blobID;
                loadConfig(mode);
                break;
            case 'script':
                break;
            case 'preview':
                break;
            default:
                
        }
    }
    //
    materialsection.dataset['mode'] = mode;
}

function resetMaterial(mode) {
    //
    materialsection.dataset['mode'] = 'none';
    //
    clearConfig(mode);
    //
    configInfo.JSON = {};
    configInfo.form = '';
    configInfo.blob = 0;
    //
    luaScript.setValue('');
    //
    luaScript.refresh();
}



//////////////////////////////////////////////
//          Configuration handling          //


//
const configForms = {
    'sprite': document.querySelector("#sprite-config"),
    'object': document.querySelector("#object-config"),
    'scene': document.querySelector("#scene-config"),
    'game': document.querySelector("#game-config"),
    'project': document.querySelector("#project-config")
};

//
let configInfo = {
    JSON: {},
    form: '',
    blob: 0
};

//
function getJSONScope(element) {
    //
    const path = [];
    //
    while (element.className != "config") {
        element = element.parentNode;
        if (element.hasAttribute("name")) {
            path.push(element.getAttribute("name"));
        }
    }
    //
    path.reverse();
    //
    let container = configInfo.JSON;
    //
    for (let key of path) {
        container = container[key];
    }
    //
    return container;
}

//
function configSetup() {
    //
    materialsection.addEventListener("input", (event) => {
        if (event.target.matches(".li-template input, #game-icon-input, :not(.json-field) > input")) {
            return;
        }
        //
        const input = event.target;
        //
        getJSONScope(input)[input.getAttribute("name")] = (input.type == "number") ? Number(input.value) : input.value;
        //
        const classToCheck = input.parentNode.className;
        //
        if ((classToCheck.includes("instance") || classToCheck.includes("reference")) && input.value == 0) {
            return;
        }
        //
        onChangeTo['config'](configInfo);
    });

    //
    materialsection.addEventListener("click", (event) => {
        //
        if (event.target.matches(":not(.plus-button, .minus-button)")) {
            return;
        }
        //
        const button = event.target;
        //
        if (button.className == "plus-button") {
            //
            const item = addItemToConfigArray(button.parentNode, getJSONScope(button).length);
            //
            if (item.className.includes("instance")) {
                return;
            }
        //
        } else {
            const li = button.parentNode;
            //
            const liname = li.querySelector(":scope [name]").getAttribute("name");
            //
            const jsonArray = li.parentNode;
            //
            jsonArray.querySelectorAll(":scope > li").forEach((item) => {
                //
                item = item.querySelector(":scope [name]");
                //
                if (parseInt(item.getAttribute("name")) <= parseInt(liname) ) {
                    return;
                }
                //
                item.setAttribute("name", parseInt(item.getAttribute("name")) - 1);
            });
            //
            getJSONScope(li).splice(parseInt(liname), 1);
            //
            li.remove();
        }
        //
        onChangeTo['config'](configInfo);
    });
}

//
function clearConfig(form) {
    //
    form = configForms[form];
    //
    if (form === undefined) {
        return;
    }
    //
    form.querySelectorAll(".json-array:not([name=\"version\"]) > li").forEach((li) => {li.remove();});
    //
    form.querySelectorAll(".json-field:not(#game-icon) > input").forEach((input) => {
        //
        if (input.matches(".li-template input")) {
            return;
        }
        //
        input.removeAttribute("value");
    });
}

//
function loadConfig(form) {
    //
    form = configForms[form];
    //
    if (form === undefined) {
        return;
    }

    //
    function determineLoadMethod(element, scope) {
        if (element.className.includes('json-field')) {
            loadVariable(element.querySelector(":scope > input"), scope, "field");
        }
        if (element.className.includes('json-object')) {
            loadVariable(element, scope, "object");
        }
        if (element.className.includes('json-array')) {
            loadVariable(element, scope, "array");
        }
    }
    //
    function loadVariable(element, scope, method) {
        //
        let key = element.getAttribute("name");
        //
        if (key === null) {
            return;
        }

        //
        if (method == "field") {
            element.value = scope[key];
            return;
        }
        //
        if (method == "object") {
            //
            element.querySelectorAll(":scope > *").forEach((attr) => {
                //
                determineLoadMethod(attr, scope[key]);
            });
        }
        //
        if (method == "array") {
            //
            let li;
            //
            const isVersion = element.getAttribute("name") == "version";
            //
            for (let i = 0 ; i < scope[key].length ; i++) {
                //
                if (isVersion) {
                    //
                    li = element.querySelector(`input[name="${i}"]`);
                    //
                    loadVariable(li, scope[key], "field");
                    continue;
                }
                //
                li = addItemToConfigArray(element, i, false);
                //
                determineLoadMethod(li, scope[key]);
            }
        }
    }
    //
    const JSON = configInfo.JSON;
    //
    form.querySelectorAll(":scope > *:not(#game-icon)").forEach((element) => {
        determineLoadMethod(element, JSON);
    });
}

//
function addItemToConfigArray(jsonArray, index, update = true) {
    //
    function determineAddMethod(element, scope) {
        if (element.className.includes('json-field')) {
            addVariable(element.querySelector(":scope > input"), scope, "field");
        }
        if (element.className.includes('json-object')) {
            addVariable(element, scope, "object");
        }
        if (element.className.includes('json-array')) {
            addVariable(element, scope, "array");
        }
    }
    //
    function addVariable(element, scope, method) {
        //
        let key = element.getAttribute("name");
        //
        if (key === null) {
            return;
        }

        //
        if (method == "field") {
            //
            scope[key] = (element.type == "number") ? Number(element.value) : element.value;
        }
        //
        if (method == "object") {
            scope[key] = {};
            //
            element.querySelectorAll(":scope > *").forEach((attr) => {
                //
                determineAddMethod(attr, scope[key]);
            });
        }
        //
        if (method == "array") {
            //
            scope[key] = [];
            //
            element.querySelectorAll(":scope > li > :is(json-field, json-object, json-array)").forEach((item) => {
                //
                determineAddMethod(item, scope[key]);
            });
        }
    }

    //
    const li = document.createElement("li");
    //
    li.innerHTML = jsonArray.querySelector(":scope > .li-template").innerHTML;
    //
    jsonArray.querySelector(":scope > .plus-button").before(li);
    //
    const licontent = li.querySelector(":scope > *");

    //
    if (licontent.className.includes('json-field')) {
        //
        const input = licontent.querySelector(":scope > input");
        //
        input.setAttribute("name", index);

        //
        if (update) {
            addVariable(input, getJSONScope(input), "field");
        }
        //
        return licontent;
    }
    //
    licontent.setAttribute("name", index);

    //
    if (update) {
        determineAddMethod(licontent, getJSONScope(licontent));
    }
    //
    return licontent;
}



///////////////////////////////////////
//          Script handling          //


//
const scriptForm = document.getElementById("lua-script");

let scriptBlob = 0;

// 
const luaScript = CodeMirror(scriptForm, {
    value: "",
    mode: "text/x-lua",
    theme: "dracula",
    tabSize: 2,
    indentUnit: 2,
	indentWithTabs: true,
	smartIndent: true,
	lineNumbers: true,
	matchBrackets: true,
    scrollbarStyle: "native"
});

//
function scriptSetup() {
    luaScript.on("change", () => {
        //
        onChangeTo['script'](luaScript.getValue(), scriptBlob);
    });
}

export { switchMaterial, resetMaterial };
export default materialSetup;