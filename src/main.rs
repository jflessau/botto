mod command;
mod matrix;
mod prelude;
use dotenv::dotenv;

use prelude::*;

use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::MigrationRunner;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = setup_db().await.unwrap_or_else(|err| {
        error!("💥 error in setting up db: {err:?}");
        std::process::exit(1);
    });

    match matrix::start_client(db).await {
        Ok(_) => {
            info!("🏁 done");
            Ok(())
        }
        Err(err) => {
            error!("💥 error in matrix client: {err:?}");
            std::process::exit(1);
        }
    }
}

async fn setup_db() -> Result<Surreal<Any>> {
    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let db = connect(db_url).await.context("fails to connect to db")?;

    db.signin(Root {
        username: &env::var("DB_USER").expect("DB_USER must be set"),
        password: &env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
    })
    .await
    .context("fails to signin")?;

    db.use_ns("default")
        .use_db("default")
        .await
        .context("fails to set namespace")?;

    MigrationRunner::new(&db)
        .up()
        .await
        .expect("Failed to apply migrations");

    Ok(db)
}
