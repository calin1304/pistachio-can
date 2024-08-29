#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;

use rocket::{Rocket, Build};
use rocket::request::FromParam;
use rocket::response::{content};
use rocket::response::status::{NotFound};
use rocket::form::{Form};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

use sha256::{digest};

use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::io;

use base64::Engine;
use base64::engine::{general_purpose};

struct PasteId(String);

impl PasteId {
    fn new(data: &str) -> Self {
        PasteId(general_purpose::STANDARD.encode(digest(data))[0..12].to_string())
    }
}

impl Display for PasteId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> FromParam<'a> for PasteId {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<PasteId, &'a str> {
        Ok(PasteId(param.to_string()))
    }
}

#[get("/")]
async fn index() -> Template {
    Template::render("index", context!{})
}

#[derive(FromForm)]
struct PasteForm<'r> {
    editor: &'r str
}

#[post("/", data="<form>")]
async fn add_paste(form: Form<PasteForm<'_>>) -> io::Result<String> {
    let paste_id = PasteId::new(&form.editor);
    let prefix = &paste_id.0[0..2];
    // TODO: Ignore file already exists I/O errors but catch all others
    let _ = std::fs::create_dir(format!("uploads/{}", prefix));
    let filepath = format!("uploads/{}/{}", prefix, paste_id);
    match File::create(filepath) {
        Ok(mut file) => file.write_all(&form.editor.as_bytes())?,
        Err(_) => ()
    };
     io::Result::Ok(paste_id.0)
}

#[get("/<id>")]
async fn get_by_id(id: PasteId) -> Result<content::RawText<File>, NotFound<String>> {
    let prefix = &id.0[0..2];
    let filename = format!("uploads/{}/{}", prefix, id);
    File::open(&filename)
        .map(|f| content::RawText(f))
        .map_err(|_| NotFound(format!("Something went wrong when trying to open file {}", filename)))
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index, get_by_id, add_paste])
        .attach(Template::fairing())
}

