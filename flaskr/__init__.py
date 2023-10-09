import os, tempfile, sqlite3, base64, json, zipfile

from flask import Flask, render_template, render_template_string, send_file, current_app, request

def create_app(test_config=None):
    # create and configure the app
    app = Flask(__name__, instance_relative_config=True)
    app.config.from_mapping(
        SECRET_KEY='dev',
        BASE_FILES_PATH=os.path.join(app.static_folder, 'base-files\\'),
    )

    if test_config is None:
        # load the instance config, if it exists, when not testing
        import tomllib
        app.config.from_file('config.toml', load=tomllib.load, text=False)
    else:
        # load the test config if passed in
        app.config.from_mapping(test_config)

    # ensure the instance folder exists
    try:
        os.makedirs(app.instance_path)
    except OSError:
        pass

    # template project files setup

    from . import base_dbs

    # give the app to the module
    base_dbs.give_app(app)
    # project template database file setup
    base_dbs.create_project_template_db()
    # item types database file setup
    base_dbs.create_type_db()

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
        with open("instance\\Export Package.zip", "rb") as prezip:
            xprt.write(prezip.read())
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