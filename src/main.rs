#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;

use rocket::{Rocket, Build};
use rocket::request::FromParam;
use rocket::response::{content};
use rocket::form::{Form};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

use sha256::{digest};

use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;

struct PasteId(String);

impl PasteId {
    fn new(data: &str) -> Self {
        PasteId(digest(data))
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
async fn add_paste(form: Form<PasteForm<'_>>) -> Result<Template, std::io::Error> {
    let paste_id = PasteId::new(&form.editor);
    let prefix = &paste_id.0[0..2];
    std::fs::create_dir(format!("uploads/{}", prefix))?;
    let filepath = format!("uploads/{}/{}", prefix, paste_id);
    let mut file = File::create(filepath)?;
    file.write_all(&form.editor.as_bytes())?;
    Ok(Template::render("index", context! { paste_id: paste_id.0 }))
}

#[get("/<id>")]
async fn get_by_id(id: PasteId) -> Option<content::RawText<File>> {
    let prefix = &id.0[0..2];
    let filename = format!("uploads/{}/{}", prefix, id);
    File::open(&filename).map(|f| content::RawText(f)).ok()
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index, get_by_id, add_paste])
        .attach(Template::fairing())
}

