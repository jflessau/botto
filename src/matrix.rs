use crate::{command::*, prelude::*};

use matrix_sdk::{
    config::SyncSettings,
    matrix_auth::MatrixSession,
    ruma::{
        events::room::{
            member::StrippedRoomMemberEvent,
            message::{MessageType, OriginalSyncRoomMessageEvent},
        },
        OwnedUserId,
    },
    Error, LoopCtrl, Room, RoomState,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::fs;

pub async fn start_client(db: Surreal<Any>) -> Result<()> {
    // create or restore session

    let sqlite_file = Path::new("client_data/sqlite_db");
    let session_file = Path::new("client_data/session.json");
    let password: String = std::env::var("BOT_PASSWORD").expect("BOT_PASSWORD must be set");

    if sqlite_file.exists() != session_file.exists() {
        warn!("expected sqlite file and session file to both exist or not exist, deleting the existing file to stat over");
        if sqlite_file.exists() {
            debug!("deleting sqlite file");
            fs::remove_file(sqlite_file)
                .await
                .context("fails to delete sqlite file")?
        } else {
            debug!("deleting session file");
            fs::remove_file(session_file)
                .await
                .context("fails to delete session file")?
        }
    }

    let (client, initial_sync_token) = if session_file.exists() {
        info!("🔍 found previous session, restoring...");
        restore_session(session_file)
            .await
            .context("fails to restore session")?
    } else {
        info!("📝 no session file found, creating new session...");
        (
            login(sqlite_file, session_file, &password)
                .await
                .context("fails to login")?,
            None as Option<String>,
        )
    };

    info!("🚀 session initialized");

    // set avatar image

    let current_avatar_file = client
        .account()
        .get_avatar_url()
        .await
        .context("fails to get avatar url")?;
    debug!("current avatar: {current_avatar_file:?}");
    if current_avatar_file.is_none() {
        info!("🏞️ no avatar image set yet, setting avatar image...");
        let image = tokio::fs::read("avatar.jpg")
            .await
            .context("fails to read avatar image file")?;
        client
            .account()
            .upload_avatar(&mime::IMAGE_JPEG, image)
            .await
            .context("fails to upload avatar image")?;
    }

    // setup event handler

    let mut sync_settings = SyncSettings::default();
    if let Some(sync_token) = initial_sync_token {
        sync_settings = sync_settings.token(sync_token);
    }

    loop {
        match client.sync_once(sync_settings.clone()).await {
            Ok(response) => {
                sync_settings = sync_settings.token(response.next_batch.clone());
                persist_sync_token(session_file, response.next_batch)
                    .await
                    .context("fails to persist sync token")?;
                break;
            }
            Err(error) => {
                error!("An error occurred during initial sync: {error}");
                info!("Trying again…");
            }
        }
    }

    info!("🦻 client listens to updates");

    let db_clone = db.clone();
    let client_clone = client.clone();

    let reminder_notifier = reminder::notify(db_clone, client_clone);

    let client_ref = &client;
    let password_ref = &password;
    let perform_cross_sign = AtomicBool::new(true);
    let perform_cross_sign_ref = &perform_cross_sign;

    let sync_client = async {
        client.add_event_handler(on_stripped_state_member);
        let db_clone = db.clone();
        client.add_event_handler(|event, room| on_room_message(event, room, db_clone));

        let Err(err) = client
            .sync_with_result_callback(sync_settings, |sync_result| async move {
                let response = sync_result?;
                persist_sync_token(session_file, response.next_batch)
                    .await
                    .map_err(|err| {
                        error!("fails to persist sync token: {err}");
                        Error::UnknownError(err.into())
                    })?;

                if let Some(user_id) = client_ref.user_id().map(|u| u.to_owned()) {
                    if perform_cross_sign_ref.swap(false, Ordering::SeqCst) {
                        tokio::spawn(bootstrap_cross_signing(
                            client_ref.clone(),
                            user_id.clone(),
                            password_ref.clone(),
                        ));

                        perform_cross_sign_ref.store(false, Ordering::SeqCst);
                    } else {
                        trace!("cross-signing was already bootstrapped");
                    }
                } else {
                    warn!("user_id is not set, cross-signing bootstrap skipped");
                }

                Ok(LoopCtrl::Continue)
            })
            .await
        else {
            bail!("sync stopped without an error, despite the library's docs stating it should only exit on error");
        };
        bail!("sync stopped: {err:?}");
    };

    let err = tokio::select! {
        err = reminder_notifier => {
            err
        }
        err = sync_client => {
            err
        }
    };

    error!("error: {err:#?}");

    Ok(())
}

async fn on_stripped_state_member(
    room_member: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
) -> Result<()> {
    let Some(user_id) = client.user_id() else {
        bail!("user_id is not set");
    };

    if room_member.state_key != user_id {
        debug!("ignoring member event for {}", room_member.state_key);
        return Ok(());
    }

    tokio::spawn(async move {
        info!("👋 joining room {}", room.room_id());
        let mut delay = 4;

        while let Err(err) = room.join().await {
            error!(
                "failed to join room {} ({err:?}), retrying in {delay} s",
                room.room_id()
            );

            tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 2048 {
                error!("fails to join room {}, error: {err:?}", room.room_id());
                break;
            }
        }
        info!("👋 joined room {}", room.room_id());
        let welcome_message = RoomMessageEventContent::text_plain("👋 Hi!\nI'm botto :)\n\nSend a message starting with '!botto' for a list of things i can do for you.");
        let _ = room
            .send(welcome_message)
            .await
            .context("fauls to send text message")
            .map_err(|err| {
                error!(
                    "fails to send welcome message to room {}, error: {err:?}",
                    room.room_id()
                );
                err
            });
    });

    Ok(())
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    db: Surreal<Any>,
) -> Result<()> {
    trace!("message from room {}, event: {:?}", room.room_id(), event);

    if room.state() != RoomState::Joined {
        debug!("ignoring message from room {}, not joined", room.room_id());
        return Ok(());
    }
    let MessageType::Text(text_content) = event.content.msgtype else {
        debug!("ignoring non-text message from room {}", room.room_id());
        return Ok(());
    };

    let text = text_content.body.trim();
    let cmd = text.to_lowercase();
    // help
    let resp = if cmd.starts_with("!botto") {
        Some(help::text())
    // conch
    } else if cmd.starts_with("!conch") {
        Some(conch::answer())
    // coinflip
    } else if cmd.starts_with("!coinflip") {
        Some(coin::flip())
    // nominate
    } else if cmd.starts_with("!nominate") {
        Some(nominate::user(&room).await)
    // choose
    } else if cmd.starts_with("!choose") {
        Some(choose::option(text.replace("!choose", "")))
    // rock, paper, scissors
    } else if cmd.starts_with("!rps") {
        Some(rps::play(text.replace("!rps", "")))
    // roll dice
    } else if cmd.starts_with("!r ") {
        Some(roll::dice(&cmd))
    // reminder
    } else if cmd.starts_with("!reminder ") {
        Some(reminder::new(room.room_id(), text, &db).await?)
    } else if cmd.starts_with("!reminders") {
        Some(reminder::list(room.room_id(), &db).await?)
    } else if cmd.starts_with("!deleteallreminder") {
        Some(reminder::delete_all(room.room_id(), &db).await?)
    } else if cmd.starts_with("!deletereminder") {
        reminder::delete(room.room_id(), text, &db).await?
    // no command
    } else {
        trace!(
            "ignoring message from room {}, no matching command",
            room.room_id()
        );
        None
    };

    if let Some(resp) = resp {
        let content = RoomMessageEventContent::text_plain(resp);
        room.send(content)
            .await
            .context("fauls to send text message")?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientSession {
    /// The URL of the homeserver of the user.
    homeserver: String,

    /// The path of the database.
    db_path: PathBuf,

    /// The passphrase of the database.
    passphrase: String,
}

/// The full session to persist.
#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    /// The data to re-build the client.
    client_session: ClientSession,

    /// The Matrix user session.
    user_session: MatrixSession,

    /// The latest sync token.
    ///
    /// It is only needed to persist it when using `Client::sync_once()` and we
    /// want to make our syncs faster by not receiving all the initial sync
    /// again.
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_token: Option<String>,
}

async fn login(sqlite_file: &Path, session_file: &Path, password: &str) -> Result<Client> {
    info!("No previous session found, logging in…");

    let (client, client_session) = build_client(sqlite_file)
        .await
        .context("fails to build client")?;
    let matrix_auth = client.matrix_auth();

    loop {
        let username: String = std::env::var("BOT_USERNAME").expect("BOT_USERNAME must be set");

        match matrix_auth
            .login_username(&username, password)
            .initial_device_display_name("persist-session client")
            .await
        {
            Ok(_) => {
                info!("Logged in as {username}");
                break;
            }
            Err(error) => {
                error!("Error logging in: {error}");
                info!("Please try again\n");
            }
        }
    }

    let Some(user_session) = matrix_auth.session() else {
        bail!("session is not set");
    };
    debug!("obtained user session");

    let serialized_session = serde_json::to_string(&FullSession {
        client_session,
        user_session,
        sync_token: None,
    })
    .context("fails to serialize session")?;
    debug!("serialied session");

    debug!("writing session to file {}", session_file.to_string_lossy());
    fs::write(session_file, serialized_session)
        .await
        .context("fails to write session file")?;

    debug!("session stored in {}", session_file.to_string_lossy());

    Ok(client)
}

async fn build_client(sqlite_file: &Path) -> anyhow::Result<(Client, ClientSession)> {
    info!("🔧 building client");
    let mut rng = thread_rng();

    let passphrase: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    loop {
        let homeserver: String = std::env::var("HOMESERVER").expect("HOMESERVER must be set");

        match Client::builder()
            .homeserver_url(&homeserver)
            .sqlite_store(sqlite_file, Some(&passphrase))
            .build()
            .await
        {
            Ok(client) => {
                return Ok((
                    client,
                    ClientSession {
                        homeserver,
                        db_path: sqlite_file.into(),
                        passphrase,
                    },
                ))
            }
            Err(error) => match &error {
                matrix_sdk::ClientBuildError::AutoDiscovery(_)
                | matrix_sdk::ClientBuildError::Url(_)
                | matrix_sdk::ClientBuildError::Http(_) => {
                    error!("Error checking the homeserver: {error}");
                    info!("Please try again\n");
                }
                _ => {
                    return Err(error.into());
                }
            },
        }
    }
}

async fn restore_session(session_file: &Path) -> anyhow::Result<(Client, Option<String>)> {
    debug!(
        "previous session found in '{}'",
        session_file.to_string_lossy()
    );

    let serialized_session = fs::read_to_string(session_file)
        .await
        .context("fails to read session file")?;
    let FullSession {
        client_session,
        user_session,
        sync_token,
    } = serde_json::from_str(&serialized_session)?;

    let client = Client::builder()
        .homeserver_url(client_session.homeserver)
        .sqlite_store(client_session.db_path, Some(&client_session.passphrase))
        .build()
        .await
        .context("fails to build client")?;

    client
        .restore_session(user_session)
        .await
        .context("fails to restore session")?;

    Ok((client, sync_token))
}

async fn persist_sync_token(session_file: &Path, sync_token: String) -> anyhow::Result<()> {
    let serialized_session = fs::read_to_string(session_file)
        .await
        .context("fails to read session file")?;
    let mut full_session: FullSession =
        serde_json::from_str(&serialized_session).context("fails to deserialize session")?;

    full_session.sync_token = Some(sync_token);
    let serialized_session =
        serde_json::to_string(&full_session).context("fails to serialize session")?;
    fs::write(session_file, serialized_session)
        .await
        .context("fails to write session file")?;

    Ok(())
}

async fn bootstrap_cross_signing(
    client: Client,
    user_id: OwnedUserId,
    password: String,
) -> Result<()> {
    info!("🔑 cross-signing");
    if let Err(e) = client.encryption().bootstrap_cross_signing(None).await {
        use matrix_sdk::ruma::api::client::uiaa;

        info!("🔑 first cross-signing attempt failed (thats expected), now retrying with password");

        if let Some(response) = e.as_uiaa_response() {
            let mut password = uiaa::Password::new(user_id.into(), password);
            password.session = response.session.clone();

            client
                .encryption()
                .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                .await
                .map_err(|e| {
                    error!("🔑 cross-signing failed: {e:?}");
                    e
                })
                .context("fails to bootstrap cross-signing")?;

            info!("🔑✅ cross-signing successful");
        } else {
            bail!("UIAA response was expected but not found");
        }

        Ok(())
    } else {
        trace!("🔑🤷 cross-signing was unnecessary");
        Ok(())
    }
}
