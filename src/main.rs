use actix_cors::Cors;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_httpauth::middleware::HttpAuthentication;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};

use schema::YodaSchema;
use yoda::Config;

async fn index(schema: web::Data<YodaSchema>, http_req: HttpRequest, req: Request) -> Response {
    let mut req = req.into_inner();
    if let Some(id) = http_req.extensions().get::<auth::Identity>().cloned() {
        req.data.insert(id);
    }
    schema.execute(req).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::new()?;

    println!("Playground: http://localhost:8000");

    let schema = config.gen_schema().await;

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .data(schema.clone())
            .service(
                web::resource("/")
                    .wrap(HttpAuthentication::bearer(auth::Validator::middleware))
                    .guard(guard::Post())
                    .to(index),
            )
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
