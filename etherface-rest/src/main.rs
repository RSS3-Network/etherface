mod v1;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use etherface_lib::database::handler::DatabaseClientPooled;
use v1::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = web::Data::new(AppState {
        dbc: DatabaseClientPooled::new().unwrap(),
    });

    HttpServer::new(move || {
        App::new().app_data(state.clone()).service(
            web::scope("/v1")
                .service(v1::signatures_by_text)
                .service(v1::signatures_by_hash)
                .service(v1::sources_github)
                .service(v1::sources_etherscan)
                .service(v1::statistics)
                .service(v1::healthcheck)
                .wrap(Cors::permissive())
                .wrap(Logger::new("(%Ts, %s) %a: %r").log_target("v1::logger")),
        )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
