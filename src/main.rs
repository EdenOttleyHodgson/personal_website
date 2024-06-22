use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use core::panic;
use lazy_static::lazy_static;
use rand::prelude::*;
use std::collections::HashMap;
use std::fs::{self, DirEntry, ReadDir};
use std::iter::empty;
use tower_http::services::ServeDir;

use axum::extract::{Query, Request};
use axum::routing as r;
use axum::{response::Html, Router};

static ROUTE_NAMES: [(&'static str, &'static str); 4] = [
    ("home", "/"),
    ("projects", "/projects"),
    ("dogs", "/dogs"),
    ("interests", "/interests"),
];

lazy_static! {
    static ref DOG_PIC_FILE_NAMES: Vec<String> = {
        match fs::read_dir("./assets/Dogs") {
            Ok(dir) => dir
                .map(|x| match x {
                    Ok(f) => Some(f.file_name().to_string_lossy().to_string()),
                    Err(_) => None,
                })
                .filter(|x| x.is_some())
                .map(|x| {
                    let s = x.unwrap();
                    s
                })
                .collect(),
            Err(e) => panic!("{}", e),
        }
    };
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/style.css", r::get(css))
        .route("/script.js", r::get(js))
        .route("/", r::get(index))
        .route("/foo", r::get(foo))
        .route("/projects", r::get(projects))
        .route("/dogs", r::get(dogs))
        .route("/interests", r::get(interests))
        .route("/navbar", r::get(navbar))
        .route("/htmx.min.js", r::get(htmx))
        .route("/test", r::get(clicked))
        .route("/getdogs", r::get(get_random_dogs))
        .nest_service("/dogpictures", ServeDir::new("assets/Dogs"));

    Ok(router.into())
}

async fn index() -> Html<String> {
    Html(fs::read_to_string("./assets/index.html").expect("index.html should exist"))
}

async fn foo() -> Html<String> {
    Html(fs::read_to_string("./assets/foo.html").expect("foo.html should exist"))
}

async fn clicked() -> Html<&'static str> {
    Html("<p> ARGHHH IT WENT BUTTON MODE! </p>")
}

async fn submit(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    Html(params.get("inp").unwrap_or(&"".to_owned()).clone())
}
async fn css() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../assets/style.css").to_owned())
        .expect("Style.css should exist")
}

async fn js() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/javascript")
        .body(include_str!("../assets/script.js").to_owned())
        .expect("script.js should be loadable")
}
async fn htmx() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/javascript")
        .body(include_str!("../assets/htmx.min.js").to_owned())
        .expect("htmx.min.js should be loadable")
}

async fn dogs() -> Html<String> {
    Html(include_str!("../assets/dogs.html").to_owned())
}
async fn interests() -> Html<String> {
    Html(include_str!("../assets/projects.html").to_owned())
}
async fn projects() -> Html<String> {
    Html(include_str!("../assets/projects.html").to_owned())
}

async fn navbar() -> Html<String> {
    let mut html_str = String::from("<div class=\"navbar\">");
    for (name, route) in ROUTE_NAMES {
        html_str.push_str(&format!(
            "<div class=\"navbar-elem\"><a class=\"navbar-elem\" href=\"{}\">{}</a></div>\n",
            route, name
        ));
    }
    html_str.push_str("</div>");
    Html(html_str)
}

async fn get_random_dogs(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let num_dogs = params
        .get("num_dogs")
        .unwrap_or(&"0".to_owned())
        .parse::<i32>()
        .unwrap_or(0);
    let mut html = String::from("<div class=\"outer-dog-div\">");

    for num in 1..=num_dogs {
        let src = get_random_dog();
        let s = format!(
            "
        <div id=\"dog-div-{}\" class=\"inner-dog\">
          <img class=\"inner-dog\" alt=\"Dog Image\" src=\"dogpictures/{}\"/>
        </div>


        ",
            num, src
        );
        html.push_str(&s);
    }

    html.push_str("</div>");

    Html(html)
}

fn get_random_dog() -> String {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..DOG_PIC_FILE_NAMES.len());
    DOG_PIC_FILE_NAMES.get(idx).unwrap().clone()
}
