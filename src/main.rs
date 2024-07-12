use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use core::panic;
use lazy_static::lazy_static;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
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
    static ref PROJECT_DATA: HashMap<String, ProjectData> = {
        match fs::read_dir("./assets/Projects") {
            Ok(dir) => dir
                // .flatten()
                .map(|x| fs::read_to_string(x.unwrap().path()).unwrap())
                // .flatten()
                .map(|f| serde_json::from_str::<ProjectData>(&f).unwrap())
                // .flatten()
                .map(|p| (p.id.clone(), p))
                .collect(),
            Err(e) => panic!("{}", e),
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProjectData {
    id: String,
    name: String,
    repo_url: String,
    description: String,
    thumbnail_url: String,
    technologies_used: Vec<String>,
}

impl Into<Html<String>> for ProjectData {
    fn into(self) -> Html<String> {
        let name_html = format!("<h2>{}</h2>", self.name);
        let repo_html = format!(
            "<a href=\"{}\"class=\"fa-brands fa-github\"></a>",
            self.repo_url
        );
        let description_html = format!("<p>{}</p>", self.description);
        let thumbnail_html = format!(
            "<img alt=\"project thumbnail\" src=\"{}\"></img>",
            self.thumbnail_url
        );
        let technologies_html = {
            let mut html = String::new();
            for technology in self.technologies_used {
                let to_push = format!("<div class=\"technology\">{technology}</div>");
                html.push_str(&to_push)
            }
            html
        };
        let final_html = format!(
            "
        <div class=\"centered-vertical-flexbox\">
            <div class=\"centered-horizontal-flexbox\">
                {name_html}
                {repo_html}
            </div>
            <div class=\"centered-horizontal-flexbox\">
                <div>
                    {description_html}
                </div>
                <div class=\"centered-vertical-flexbox\">
                    {thumbnail_html}
                    <div class=\"centered-vertical-flexbox\">
                        {technologies_html}
                    </div>
                </div>
            </div>
        </div>
        "
        );

        Html(final_html)
    }
}
impl ProjectData {
    fn button_html(&self) -> String {
        let vals = format!("js:{{id: '{}'}}", self.id);
        let out = format!(
            "<button hx-get=\"get_project_data\" hx-vals=\"{}\" hx-target=\"#project-info\" hx-swap=\"innerHTML\"class=\"project-selector\">{}</button>",
            vals, self.name
        );
        println!("{}", out);
        out
    }
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
        .route("/get_project_data", r::get(get_project_data))
        .route("/project_selector", r::get(project_selector))
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

async fn get_project_data(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let id = if let Some(id) = params.get("id") {
        id
    } else {
        println!("Bad request for project data!");
        return Html("<p> bad request! </p>".to_owned());
    };
    PROJECT_DATA
        .get(id)
        .map(|f| {
            let h: Html<String> = Into::into(f.clone());
            h
        })
        .unwrap_or(Html("<p>Project not Found!</p>".to_owned()))
}

async fn project_selector() -> Html<String> {
    let mut html = String::from("<div class=\"centered-vertical-flexbox project_selector_box\">");
    for (_, project) in PROJECT_DATA.iter() {
        html.push_str(&project.button_html())
    }
    html.push_str("</div>");
    Html(html)
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
