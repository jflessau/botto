use crate::prelude::*;
use matrix_sdk::{room::Room, RoomMemberships};
use rand::seq::SliceRandom;
use std::env;

pub async fn user(room: &Room) -> String {
    let own_username = env::var("BOT_USERNAME").expect("BOT_USERNAME must be set");
    let members = room.members(RoomMemberships::ACTIVE).await;

    match members {
        Ok(m) => {
            let m = m
                .into_iter()
                .filter(|m| m.name() != own_username)
                .map(|m| {
                    let mut s = m.display_name().unwrap_or(m.name()).to_string();
                    if m.name_ambiguous() {
                        s = format!("{s} ({})", m.user_id())
                    }
                    s
                })
                .collect::<Vec<_>>();

            debug!("members: {m:?}");

            m.choose(&mut rand::thread_rng())
                .map(|n| n.to_string())
                .unwrap_or_else(|| {
                    error!("fails to select nominee");
                    "Couldn't find an active user :(".into()
                })
        }
        Err(err) => {
            error!("failed to get members: {err:?}");
            "Couldn't find an active user :(".to_string()
        }
    }
}
