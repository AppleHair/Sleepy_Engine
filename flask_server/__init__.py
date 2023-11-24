import os, tempfile, sqlite3, base64, json, zipfile, tomllib

from flask import Flask, render_template, render_template_string, send_file, current_app, request

CONFIG_FILE = """# Loading booleans
LOAD_BASE_FILES=false
LOAD_EXPORT_PACKAGE=false
# Loading paths
BASE_FILES_PATH='flask_server\\static\\base-files\\'
ENGINE_CORE_PATH='flask_server\\static\\js\\engine-core\\'
CORE_SOURCE_PATH='game-engine\\'
EXPORT_PACKAGE_PATH='instance\\Export Package.zip'
JS_LIBRARYS_PATH='flask_server\\static\\js\\libs\\'"""

def create_app(test_config=None):
    # create and configure the app
    app = Flask(__name__, instance_relative_config=True)
    # ensure the instance folder exists
    try:
        os.makedirs(app.instance_path)
        open('instance\\config.toml', 'w').write(CONFIG_FILE)
        app.config.from_mapping(
            LOAD_BASE_FILES=True,
            LOAD_EXPORT_PACKAGE=True,
            BASE_FILES_PATH='flask_server\\static\\base-files\\',
            ENGINE_CORE_PATH='flask_server\\static\\js\\engine-core\\',
            CORE_SOURCE_PATH='game-engine\\',
            EXPORT_PACKAGE_PATH='instance\\Export Package.zip',
            JS_LIBRARYS_PATH='flask_server\\static\\js\\libs\\',
        )
    except OSError:
        app.config.from_file('config.toml', load=tomllib.load, text=False)
        pass

    # template project files setup

    if app.config['LOAD_BASE_FILES']:

        from . import base_dbs
        # give the app to the module
        base_dbs.give_app(app)
        # project template database file setup
        base_dbs.create_project_template_db()
        # item types database file setup
        base_dbs.create_type_db()

    # export package setup

    if app.config['LOAD_EXPORT_PACKAGE']:
        pre_xprt = None
        xprt_zip = None
        try:
            pre_xprt = open(app.config['EXPORT_PACKAGE_PATH'], "wb")
            xprt_zip = zipfile.ZipFile(pre_xprt, 'w')
        except FileExistsError:
            pre_xprt = open(app.config['EXPORT_PACKAGE_PATH'], "xb")
            xprt_zip = zipfile.ZipFile(pre_xprt, 'x')
            pass
        
        xprt_zip.write(app.config['CORE_SOURCE_PATH']+'load-game.js', 'load-game.js')
        xprt_zip.write(app.config['JS_LIBRARYS_PATH']+'sql-wasm.js', 'sql-wasm.js')
        xprt_zip.write(app.config['JS_LIBRARYS_PATH']+'sql-wasm.wasm', 'sql-wasm.wasm')
        xprt_zip.write(app.config['ENGINE_CORE_PATH']+'game_engine.js', 'game_engine.js')
        xprt_zip.write(app.config['ENGINE_CORE_PATH']+'game_engine_bg.wasm', 'game_engine_bg.wasm')
        xprt_zip.close()
        pre_xprt.close()

    # base route
    @app.route('/', methods=['GET'])
    def open_editor():
        return render_template('editor.html')
    # game test route
    @app.route('/game-test', methods=['GET'])
    def open_game_test():
        return render_template('game-test.html')
    # game export route
    @app.route('/export', methods=['POST'])
    def export_game():
        # connect to the received database
        conn = sqlite3.connect(":memory:")
        conn.deserialize(request.files['gameData'].read())
        # extract the game icon and game title
        cur = conn.cursor()
        cur.execute("SELECT data FROM blobs WHERE rowid=3;")
        gameIcon = "data:image/x-icon;base64," + str(base64.standard_b64encode(cur.fetchone()[0]))[2:-1]
        cur.execute("SELECT data FROM blobs WHERE rowid=1;")
        gameTitle = json.loads(cur.fetchone()[0])["browser-title"]
        cur.close()
        # copy the pre-structured export package to a temp file
        xprt = tempfile.TemporaryFile()
        with open(app.config['EXPORT_PACKAGE_PATH'], "rb") as pre_xprt:
            xprt.write(pre_xprt.read())
        # add the received database and a 
        # rendered web page, with the game
        # icon and title, to the package
        xprt_zip = zipfile.ZipFile(xprt, 'a')
        xprt_zip.writestr("index.html", render_template('game-export.html', title = gameTitle, icon = gameIcon).encode('utf8'))
        xprt_zip.writestr("data.sqlite", conn.serialize())
        xprt_zip.close()
        conn.close()
        # return the export package to the user
        xprt.seek(0)
        return send_file(xprt, "application/zip")

    return app