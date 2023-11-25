////////////////////////////////////////////
//          Pop-up Window module          //
////////////////////////////////////////////

// MDN web docs - Document Object Model:
// https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
// https://developer.mozilla.org/en-US/docs/Web/API/HTML_DOM_API


// This is a pointer to the pop-up HTML element
const popup = document.querySelector(".page-wrapper > .pop-up");


// This is a pointer to the pop-up window HTML element
const popupWin = popup.querySelector(":scope > .window");
// This is a pointer to the window's message p HTML element
const popupMessage = popupWin.querySelector(":scope > p");

// This is an object which contains
// pointers to different input fields
// in the pop-up window
const popupInput = {
    // This is a pointer to the name input HTML element
    'name': popupWin.querySelector(":scope #name"),
    // This is a pointer to the color input HTML element
    'color': popupWin.querySelector(":scope #color"),
    // This is a pointer to the type select HTML element
    'type': popupWin.querySelector(":scope #type"),
    // This is a pointer to the data input HTML element
    'data': popupWin.querySelector(":scope #data")
}
// This is an object which contains
// default values for different fields
// in the pop-up window
const popupDefaultValues = {
    'name': '',
    'color': '#70e65c',
    'data': ''
}

// These are pointers to the 
// confirm and cancal HTML
// HTML elements of the pop-up window
const popupConfirm = popupWin.querySelector(":scope #confirm");
const popupCancal = popupWin.querySelector(":scope #cancal");

// This function closes the 
// pop-up window and disables 
// all of its fields
function closePopupWindow() {
    // We empty the innerText
    // of the message
    popupMessage.innerText = "";
    // We make the window's 
    // buttons do nothing when
    // they are clicked
    popupCancal.onclick = null;
    popupConfirm.onclick = null;
    // We remove the pop-up HTML element's
    // style attribute and make it invisible
    popup.removeAttribute("style");

    // We itrate on the pop-up 
    // window's input fields
    for (let input in popupInput) {
        // if the input field has a default value
        if (popupDefaultValues[input] !== undefined) {
            // we give it its default value
            popupInput[input].value = popupDefaultValues[input];
        }
        // We disable the input field and remove
        // its style attribute, which makes it
        // invisible and unusable
        popupInput[input].setAttribute("disabled","");
        popupInput[input].parentNode.removeAttribute("style");
    }

    // We remove all the options out of the type select HTML element
    popupInput['type'].querySelectorAll(":scope > *").forEach((item) => item.remove());
}

// This function received a message string
// and an onConfirm function. It's used
// for displaying a message to the user
// before preforming a certain action, 
// asking him to confirm the action. 
function openMessageWindow(message, onConfirm) {
    // We set the pop-up HTML element's
    // display property to flex, in
    // order to make it and its content
    // appear correctly on the page
    popup.style.display = "flex";
    // We set the innerText of the
    // pop-up message to the message
    // received
    popupMessage.innerText = message;
    // We make the cancal button close
    // the window when it's clicked
    popupCancal.onclick = () => {
        closePopupWindow();
    }
    // We make the confirm button
    // run the onConfirm function
    // and close the window when
    // it's clicked
    popupConfirm.onclick = () => {
        onConfirm();
        closePopupWindow();
    }
}

// This function receives a message
// string, an array of input field
// names, an onConfirm function, a
// types map and a row object. It's
// used to let the user create or 
// edit items by setting the name,
// color, type or data fields from
// the pop-up window.
function openInputWindow(message, inputs, onConfirm, types, row) {
    // We set the pop-up HTML element's
    // display property to flex, in
    // order to make it and its content
    // appear correctly on the page
    popup.style.display = "flex";
    // We set the innerText of the
    // pop-up message to the message
    // received
    popupMessage.innerText = message;
    // We itrate on the received
    // input field names
    for (let input of inputs) {
        // We get the pointer to the input
        // field with the received name
        input = popupInput[input];
        // if theres no input field
        // with the received name, 
        // we continue the itration
        if (input === undefined) {
            continue;
        }
        // We set the HTML element's
        // display property to flex
        // and remove its disabled 
        // attribute to make it visible 
        // and usable
        input.removeAttribute("disabled");
        input.parentNode.style.display = "flex";

        // Because the row's 'data' field
        // only contains a rowid to the
        // blobs table, which is out of
        // our reach, we can't get the
        // value of 'data' from it
        if (input.id == "data") {
            continue;
        }

        // if we received an existing
        // row that we should edit
        if (row !== undefined) {
            // We receive its corresponding
            // value to the input HTML element
            let curValue = row[input.id];
            // if he has such value
            if (curValue !== undefined) {
                // we give it to the 
                // input HTML element
                input.value = curValue;  
            }
        }
    }

    // if we received a types map
    if (types !== undefined) {
        // we itrate on its key, value pairs
        types.forEach((value,key) => {
            // We add an option to the type select HTML element
            const option = popupInput['type'].appendChild(document.createElement("option"));
            // We set this option's value to the key
            option.setAttribute("value", key);
            // We set this option's innerText to the value
            // (with a capital letter at the start)
            option.innerText = value.charAt(0).toUpperCase() + value.substring(1);
        });
    }
    // We make the cancal button close
    // the window when it's clicked
    popupCancal.onclick = () => {
        closePopupWindow();
    }
    // We make the confirm button
    // run the onConfirm function
    // with a results array, containing 
    // the input values received from
    // the user, and close the window 
    // when it's clicked
    popupConfirm.onclick = async () => {
        // if the name input field is empty
        if (popupInput['name'].value == '' && inputs.includes('name')) {
            // we alert the user and return
            // (the window won't close)
            alert(`Failed Requirement:
                'Name' must have at least 1 character.`);
            return;
        }
        // if the data input field is empty
        if (popupInput['data'].value == '' && inputs.includes('data')) {
            // we alert the user and return
            // (the window won't close)
            alert(`Failed Requirement:
                'Asset File' must be loaded.`);
            return;
        }
        // We diclare the results array
        let results = [];
        // We itrate on the received
        // input field names
        for (let input of inputs) {
            // We get the pointer to the input
            // field with the received name
            input = popupInput[input];
            // if theres no input field
            // with the received name, 
            // we push null to the results
            // array and continue the itration
            if (input === undefined) {
                results.push(null);
                continue;
            }
            // if the input field is
            // the data field, We push
            // its file object to the
            // results and continue the
            // itration
            if (input.id == "data") {
                results.push(input.files[0]);
                continue;
            }
            // we push the input field's
            // value into the results array
            results.push(input.value);
        }
        // We run the onConfirm function
        // with the results array
        await onConfirm(results);
        // We close the window
        closePopupWindow();
    }
}

export { openMessageWindow, openInputWindow };