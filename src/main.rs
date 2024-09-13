mod command;
mod matrix;
mod prelude;

use prelude::*;

use surrealdb::{
    engine::any::{connect, Any},
    opt::auth::Root,
    Surreal,
};
use surrealdb_migrations::MigrationRunner;

#[tokio::main]
async fn main() -> Result<()> {
    let db = setup_db().await.unwrap_or_else(|err| {
        error!("💥 error in db setup: {err:?}");
        std::process::exit(1);
    });

    db.query("fn::create_reminder('test_room', 'test-reminder', 'minute', 1, none, true)")
        .await?;

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
    let db = connect("ws://localhost:8000")
        .await
        .context("fails to connect to db")?;

    db.signin(Root {
        username: "root",
        password: "root",
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
