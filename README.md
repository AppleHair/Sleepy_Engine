# Repository Structure (in a nutshell)
This repository is built out of several code bases:
## 1) Flask server ("flask_server" directory)
This is the server that runs the editor website. It's built using Python and Flask.
The server also creates and manages SQLite databases, which are used as templates for the project files, and exports the project files to a "html archive" which includes an html page, which uses other files in the archive, including the project files, to run the game in the browser.
## 2) The editor's code ("flask_server/static/js/editor" directory)
This is the main code of the editor. It's built using JavaScript and interacts with the server and SQLite databases (project files). It also uses a special "game-test" page to test the edited game in the browser.
## 3) The game engine's core ("game-engine" directory)
This is the core of the game engine, where all the systems and API components get implemented. It's built using Rust and compiles to WebAssembly. It's used by the "game-test" page and the game export, which is made by the server, to run the game in the browser.
# Setting Up The Environment
Before running the server, you need to set up the environment. To do so, follow these steps in order:
## 1) Install Python (3.12)
Go to https://www.python.org/downloads/ and Install The Python 3.12 on your machine.

NOTE: Make sure to check the box that says "Add Python 3.12 to PATH"
## 2) Install Rust and wasm-pack
Go to https://www.rust-lang.org/tools/install and follow the instructions to install Rust on your machine. After installing Rust, run the following command in your terminal to install wasm-pack:
```
cargo install wasm-pack
```
## 3) Fill Librarys Directory
At "flask_server/static/js", there's an empty directory called "libs". Follow the instructions in the other read me file inside it to fill the directory with the necessary files.
## 4) Run "Setup_Python_Environment"(.bat or .bash)
## 5) OPTIONAL: Run "Compile_n_Copy"(.bat or .bash) to compile the core and copy it to ../js/engine-core
## 6) Run The Server With "Run_Flask_App"(.bat or .bash)

