use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use base64::{decode, encode};
use ocastaproxy::rewrite;
use serde::Deserialize;

#[derive(Deserialize)]
struct FormData {
    url: String,
}

#[derive(Deserialize)]
struct UrlData {
    encoding: String,
    url: String,
}

#[derive(Deserialize)]
struct ProxyData {
    #[serde(flatten)]
    query: std::collections::HashMap<String, String>,
}

async fn index() -> impl Responder {
    return HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"));
}

#[get("/{encoding}/gateway")]
async fn gateway(data: web::Query<FormData>, path: web::Path<String>) -> impl Responder {
    let mut url = data.url.clone();
    if !data.url.starts_with("http") {
        url = format!("https://{}", data.url);
    }
    url = match path.as_str() {
        "b64" => encode(url),
        _ => url,
    };

    return HttpResponse::Found()
        .append_header(("Location", format!("/{}/{}", path, url)))
        .finish();
}

#[get("/{encoding}/{url:.*}")]
async fn proxy(
    path: web::Path<UrlData>,
    query: web::Query<ProxyData>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let query = query
        .query
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");
    let url = match path.encoding.as_str() {
        "b64" => decode(path.url.clone())?,
        _ => path.url.clone().into_bytes(),
    };
    let url = reqwest::Url::parse(&String::from_utf8(url)?)?;
    let new_url = reqwest::Url::parse(&format!("{}?{}", url, query))?;
    let page = reqwest::get(new_url.as_ref()).await?.text().await?;
    let new_page = rewrite::html(page, url, path.encoding.clone());

    return Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(new_page));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .service(gateway)
            .service(proxy)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
