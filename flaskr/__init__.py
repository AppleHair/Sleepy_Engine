import os, firebase_admin

from flask import Flask, render_template

def create_app(test_config=None):
    # create and configure the app
    app = Flask(__name__, instance_relative_config=True)
    app.config.from_mapping(
        SECRET_KEY='dev',
        FIREBASE_KEY_PATH=os.path.join(app.instance_path, 'key.json'),
        BASE_FILES_PATH=os.path.join(app.static_folder, 'base-files'),
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

    # certificate firebase credentials
    from firebase_admin import credentials
    cred = credentials.Certificate(app.config['FIREBASE_KEY_PATH'])
    # initialize firebase app with admin credentials
    FB_Admin = firebase_admin.initialize_app(cred)
    # receive firestore client
    from firebase_admin import firestore
    db = firestore.client()

    # base databases setup

    from . import base_dbs

    # give the app to the module
    base_dbs.give_app(app)
    # project template database file setup
    base_dbs.create_project_template_db()
    # item types database file setup
    base_dbs.create_type_db()

    # base route
    @app.route('/')
    def open_editor():
        return render_template('editor.html')

    return app