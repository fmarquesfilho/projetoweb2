//! src/startup.rs
use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::*;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres.");

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.with_db())
        .await
}

fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
            .route("/ping", web::get().to(ping))
            .route("/subscriptions", web::post().to(subscribe))
            // /cart
            .route("/cart", web::post().to(create_cart))
            .route("/cart/products", web::post().to(add_product_to_cart))
            // /users
            .route("/users", web::post().to(create_user))
            .route("/users", web::put().to(update_user))
            .route("/users", web::get().to(get_all_users))
            .route("/users/{id}", web::get().to(get_user_by_id))
            .route("/users/{id}", web::delete().to(delete_user))
            // products
            .route("/products", web::post().to(create_product))
            // app data
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
