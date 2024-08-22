#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;

use rocket::{Rocket, Build};
use rocket::request::FromParam;
use rocket::response::{content};
use rocket_dyn_templates::Template;

use sha256::{digest};

use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;

struct PasteId(String);

impl PasteId {
    fn new(data: &String) -> Self {
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
    let context = context! { name: "Regular user" };
    Template::render("index", &context)
}

#[post("/", data="<body>")]
async fn add_paste(body: String) -> Result<(), std::io::Error> {
    let paste_id = PasteId::new(&body);
    let prefix = &paste_id.0[0..2];
    std::fs::create_dir(format!("uploads/{}", prefix))?;
    let filepath = format!("uploads/{}/{}", prefix, paste_id);
    let mut file = File::create(filepath)?;
    file.write_all(&body.as_bytes())
}

#[get("/<id>")]
async fn get_by_id(id: PasteId) -> Option<content::RawText<File>> {
    let filename = format!("uploads/{id}", id = id);
    File::open(&filename).map(|f| content::RawText(f)).ok()
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, get_by_id, add_paste])
}

