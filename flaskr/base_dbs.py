import sqlite3, os

current_app = None

# for importing binarys of base files
def import_basefile(filename):
    if current_app == None:
        return
    with open(current_app.config['BASE_FILES_PATH']+filename, "rb") as file:
        data = file.read()
    return data

def give_app(app):
    global current_app 
    current_app = app

# creates the 'new project' file template in 
# the static folder with the other base files
def create_project_template_db():
    if current_app == None:
        return
    db = sqlite3.connect(os.path.join(current_app.config['BASE_FILES_PATH'], 'new-project.sqlite'))
    cur = db.cursor()
    cur.executescript(current_app.open_resource('new-project.sql').read().decode('utf8'))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("stateConfig.json"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("stateScript.rhai"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("gameIcon.ico"),))

    db.commit()
    cur.close()
    db.close()

# creates the item types database file in 
# the static folder with the other base files
def create_type_db():
    if current_app == None:
        return
    db = sqlite3.connect(os.path.join(current_app.config['BASE_FILES_PATH'], 'type.sqlite'))
    cur = db.cursor()
    cur.executescript(current_app.open_resource('type.sql').read().decode('utf8'))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("objectScript.rhai"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("objectConfig.json"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("sceneScript.rhai"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("sceneConfig.json"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("spriteConfig.json"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("audioConfig.json"),))
    cur.execute("INSERT INTO blobs VALUES (?)", (import_basefile("fontConfig.json"),))

    db.commit()
    cur.close()
    db.close()