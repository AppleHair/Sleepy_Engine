//////////////////////////////////////////
//          SQL Control Module          //
//////////////////////////////////////////

// This module uses sql.js to load, query and modify sqlite databases in the browser.
// The comments in this module describe the way various methods and classes
// from sql.js have been used in this script, and descriptions of the
// methods themselves are provided by links to the sql.js documentation,
// which are placed in the relevant places throughout the script.

// SqlJs object description: https://sql.js.org/documentation/global.html#SqlJs
// initSqlJs method description: https://sql.js.org/documentation/global.html#initSqlJs
// Database class description: https://sql.js.org/documentation/Database.html


// This is a SqlJs object, and its
// purpose is to create Database objects.
// It's defined using the initSqlJs method
// in the init function.
let sqlite;

// This is the array which
// contains all of the server
// database request objects.
// The init function
// iterate on this array and
// loads all of the requested
// server databases. Additionally,
// the init will clear this array
// and the objects it points to from 
// memory by diclaring it `undefined`,
// so this array will not be usable after
// the init function finishes.
let serverDBRequests = [];

// This class defines the server
// database request objects,
// which give instructions for
// loading SQLite databases  
// that are stored in a server.
class ServerDBRequest {
    // This constructor will never return
    // `this`, because objects created with
    // this class will always be pointed at
    // only by the serverDBRequests array,
    // and that's because we don't want to
    // prevent the GC from clearing every
    // object of this class from memory
    // when the init function finishes, 
    // and the serverDBRequests array
    // becomes undefined.
    constructor(url, onLoad) {
        if (serverDBRequests === undefined) {
            // the serverDBRequests will become
            // undefined after the init
            // function finishes, so we need
            // to make sure it's not being 
            // read when it's undefined.
            return { no: null };
        }
        // every object contains a url
        // that leads to the requested
        // database, and an onLoad function,
        // which is being executed after the
        // database has loaded, and gets the
        // database's object as an argument.
        this.url = url;
        this.onLoad = onLoad;
        // every object from this class will
        // be inserted to the serverDBRequests
        // array, which will be iterated on
        // by the init function.
        serverDBRequests.push(this);
        // we don't return `this` in order
        // to prevent variables other than
        // serverDBRequests from pointing
        // to objects from this class.
        return { no: null };
    }
}



//////////////////////////////////////////////////
//          Database Loading Functions          //


// this function should be used to initiate
// all SQLite database loading functionality
// on the script, and load all the server 
// databases with it. 
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
    // into it, with the correct path to the .wasm
    // file on the server.
    sqlite = await initSqlJs({locateFile: (file) => `/static/js/libs/${file}`});
    // We iterate on all the ServerDBRequest objects
    for (let request of serverDBRequests) {
        // We use fetch to load the database file from the server
        // while giving it the url from the serverDBRequest object.
        // We then get the file's binary data buffer and convert it into
        // an array of unsigned 8-bit integers. We then give the new
        // array to the Database object constructor, and it creates
        // a new Database object, which we give to the onLoad function
        // from the serverDBRequest object.
        request.onLoad(new sqlite.Database(new Uint8Array(await fetch(request.url).then((res) => res.arrayBuffer()))));
    }
    // We make serverDBRequests undefined in order to free
    // every ServerDBRequest object from memory and make
    // the class unusable.
    serverDBRequests = undefined;
    // We return true to indicate that
    // the server databases loaded.
    return true;
}

// This function receives a Uint8Array,
// which represents the contents of a
// SQLite database file. It returns
// a new database object, which contains
// copied data from the the database file
// which was stored in the array.

// Warning: This function will not work
// if the init function wasn't finished,
// because this function uses the sqlite
// object to create database object, and 
// that sqlite object is defined by the
// init function asynchronously. Therefore,
// this function should only be used in the
// following scope: init().then((loaded) => {*scope*});
function loadFromUint8Array(arr) {
    return new sqlite.Database(arr);
}



//////////////////////////////////////////////////
//          Database Querying functions         //


// Database.exec method description: `https://sql.js.org/documentation/Database.html#%5B"exec"%5D`
// Database.each method description: `https://sql.js.org/documentation/Database.html#%5B"each"%5D`
// Database.QueryExecResult object description: https://sql.js.org/documentation/Database.html#.QueryExecResult

// Statement class description: https://sql.js.org/documentation/Statement.html
// Database.prepare method description: `https://sql.js.org/documentation/Database.html#%5B"prepare"%5D`
// Statement.step method description: `https://sql.js.org/documentation/Statement.html#%5B"step"%5D`
// Statement.getAsObject method description: `https://sql.js.org/documentation/Statement.html#%5B"getAsObject"%5D`
// Statement.free method description: `https://sql.js.org/documentation/Statement.html#%5B"free"%5D`


// This function receives a database
// object and a table name. Using these
// arguments, it returns a QueryExecResult
// object, which represents the contents of
// the requested table in the requested database.
function getTable(db, table) {
    return db.exec(`SELECT rowid, * FROM ${table};`)[0];
}

// This function receives a database
// object and a table name. It returns
// the length of the received table,
// and by that I mean the amount of columns
// in the table, not including the rowid column.
function getTableLength(db, table) {
    let stmt = db.prepare(`SELECT * FROM ${table} LIMIT 1;`);
    stmt.step();
    let res = stmt.getColumnNames();
    stmt.free();
    return res.length;
}

// This function receives a database
// object, a table name, and a callback
// function. Using these arguments, it
// iterates on each row in the requested
// table, and calls the callback function
// every iteration, giving it the row's object
// as the first argument.
function forEachInTable(db, table, callback) {
    db.each(`SELECT rowid, * FROM ${table};`, [], callback);
}

// This function receives a database
// object, a table name, and a rowid.
// Using these arguments, it finds the
// correct row from the requested table,
// which has the requested rowid, and
// returns it as an object.
function getRow(db, table, rowid) {
    let stmt = db.prepare(`SELECT rowid, * FROM ${table} WHERE rowid=?;`, [rowid]);
    stmt.step();
    let res = stmt.getAsObject();
    stmt.free();
    return res;
}

// This function receives a database
// object, a table name, a keycolumn name
// and a valuecolumn name. Using these
// arguments, it returns a Map, which
// uses values from the keycolumn in
// the requested table as keys, and the
// corresponding values from the valuecolumn 
// in the requested table as values.
function getMap(db, table, keycolumn, valuecolumn) {
    const res = new Map();
    db.each(`SELECT ${keycolumn},${valuecolumn} FROM ${table};`, [],
            (row) => res.set(row[`${keycolumn}`], row[`${valuecolumn}`]));
    return res;
}



///////////////////////////////////////////////////
//          Database Modifying functions         //


// This function receives a database
// object, a table name and an array
// of values. It trys to add a row
// with the received values to the
// received table, and if it succeeds,
// it returns an object which represents
// the row that was just added, but 
// returns null otherwise.

// In order for this function to work,
// the length of the values array and
// the order of the values in the array
// should match to the order and amount
// of columns in the received table (not
// including the rowid column). 
function addRow(db, table, values) {
    // if the length of the table
    // doesn't match the length of 
    // the values array, we return null.
    if (getTableLength(db, table) != values.length) {
        return null;
    }
    // We use the length of
    // the values array to 
    // add the correct amount of
    // parameters to the sql statment.
    let str = '?,'.
        repeat(values.length).
        substring(0,(values.length*2)-1);
    
    // We try to insert the received
    // values to the received table.
    db.exec(`INSERT INTO ${table} VALUES(${str});`, values);
    
    // if an error didn't occur,
    // we can continue and return
    // the object of the inserted row.

    // We use SQLite's built-in last_insert_rowid()
    // function to find the last inserted row in the
    // received table, which is the one we just inserted.
    let stmt = db.prepare(`SELECT rowid, * FROM ${table} WHERE rowid=last_insert_rowid();`);
    stmt.step();
    let res = stmt.getAsObject();
    stmt.free();

    // We return the resulting object.
    return res;
}

// This function receives a database
// object, a table name and a row id.
// It trys to delete the row with the
// received row id from the received
// table, assuming the received arguments
// are accurate to the current state of
// the database.
function deleteRow(db, table, rowid) {
    db.exec(`DELETE FROM ${table} WHERE rowid=?;`, [rowid]);
}

// This function receives a database
// object, a table name, a row id,
// a column name and a value. It trys
// to update the received column in the
// received row in the received table to
// the received value, assuming the received
// arguments are accurate to the current
// state of the database.
function updateRowValue(db, table, rowid, column, value) {
    db.exec(`UPDATE ${table} SET ${column}=? WHERE rowid=?;`, [value, rowid]);
}


// We export the things we want the
// importing script to be able to use.

export { ServerDBRequest, loadFromUint8Array, getTable, getMap, forEachInTable, getRow, addRow, deleteRow, updateRowValue };
export default init;