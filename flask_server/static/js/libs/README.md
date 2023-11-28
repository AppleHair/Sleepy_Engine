## 1) Download the necessary files
You need to download codemirror 5.65.13's source code,
rhai playground's source code and SQL.js 1.8.0 "wasm-version".
(codemirror: https://github.com/codemirror/codemirror5/releases/tag/5.65.13)
(rhai playground: https://github.com/rhaiscript/playground)
(SQL.js: https://github.com/sql-js/sql.js/releases/tag/v1.8.0)

## 2) Fill this directory according to the following instructions
From codemirror's source code, take "src\codemirror.js"
and "lib\codemirror.css" and put them inside "static\js\libs\codemirror-5.65.13\lib".
Then take the "theme" and "addon" folders with their contents
and copy them into "static\js\libs\codemirror-5.65.13".

From rhai playground's source code, copy "js\wasm_loader.js"
and put it inside "static\js\mode\rhai-playground". Then,
compile the playground's source code using wasm-pack 
(with the command ```wasm-pack build --target web```)
and copy the generated "pkg\web_wasm.js" and
"pkg\web_wasm_bg.wasm" files into "static\js\mode\rhai-playground".

Finally, from SQL.js, Take the "sql-wasm.js" and "sql-wasm.wasm"
files and put them inside "static\js\libs".

Everything else should already be in place.
