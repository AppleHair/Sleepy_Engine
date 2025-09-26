## 1) Download the necessary files
You need to download codemirror 5's source code,
rhai playground's source code and SQL.js 1.8.0 "wasm-version".
(codemirror: https://codemirror.net/5/codemirror.zip)
(rhai playground: https://github.com/rhaiscript/playground)
(SQL.js: https://github.com/sql-js/sql.js/releases/tag/v1.8.0)

## 2) Fill this directory according to the following instructions
From SQL.js, Take the "sql-wasm.js" and "sql-wasm.wasm"
files and put them inside "static\js\libs".

From codemirror's source code, copy the "theme", "addon"
and "lib" folders with their contents into "static\js\libs\codemirror5".

Compile the playground's source code using wasm-pack 
(with the command ```wasm-pack build --target web```)
and copy the generated "pkg\rhai_playground.js" and
"pkg\rhai_playground_bg.wasm" files into "static\js\mode\rhai-playground".
