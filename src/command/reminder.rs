use crate::prelude::*;
use regex::Regex;
use std::cmp::Ordering;
use tokio::time::{sleep, Duration as TokioDuration};

pub async fn new(room_id: &RoomId, text: &str, db: &Surreal<Any>) -> Result<String> {
    match Reminder::try_from_str(text, room_id) {
        Ok(reminder) => {
            let db_res: Vec<Reminder> = db.create("reminder").content(&reminder).await?;
            info!("⏲️ reminder created: {db_res:?}");
            Ok(format!("Reminder created: {}", reminder))
        }
        Err(err) => {
            warn!("fails to parse reminder from {text}, error: {err:?}");
            Ok("Sorry, I don't know how to parse that reminder.\nUse the !botto command to get some hints.".to_string())
        }
    }
}

pub async fn list(room_id: &RoomId, db: &Surreal<Any>) -> Result<String> {
    let reminders: Vec<Reminder> = db
        .query("select * from reminder where room_id = $room_id order by created_at asc")
        .bind(("room_id", room_id.to_string()))
        .await?
        .take(0)?;

    if reminders.is_empty() {
        return Ok(
            "No reminders found. Create one with e.g.\n\n!reminder 2 days: Take out the trash.\n\nOr use the !botto command to get more info."
                .to_string(),
        );
    }

    info!("⏲️ list {} reminders", reminders.len());

    let mut res = "⏲️ Reminders:".to_string();
    for (n, r) in reminders.iter().enumerate() {
        res.push_str(&format!("\n{}. {r}", n + 1));
    }

    Ok(res)
}

pub async fn delete_all(room_id: &RoomId, db: &Surreal<Any>) -> Result<String> {
    db.query("delete from reminder where room_id = $room_id")
        .bind(("room_id", room_id.to_string()))
        .await?;

    info!("⏲️🗑️ All reminders deleted for room: {room_id}");

    Ok("All reminders deleted.".to_string())
}

pub async fn delete(room_id: &RoomId, text: &str, db: &Surreal<Any>) -> Result<Option<String>> {
    let re = Regex::new(r"([0-9]+)")?;

    let Some(index) = re
        .find_iter(text)
        .next()
        .and_then(|m| m.as_str().parse::<usize>().ok())
    else {
        warn!("fails to parse index from text: {text}");
        return Ok(None);
    };

    debug!("index: {index:?}");

    let reminders: Vec<Reminder> = db
        .query("select * from reminder where room_id = $room_id order by created_at asc")
        .bind(("room_id", room_id.to_string()))
        .await?
        .take(0)?;

    debug!("reminder_ids: {reminders:#?}");

    let Some(reminder) = reminders.get(index.saturating_sub(1)) else {
        debug!(
            "tried to get reminder by index: {index}, but max index is {}",
            reminders.len()
        );
        return Ok(None);
    };

    let reminder: Option<Reminder> = db.delete(&reminder.id).await?;

    if let Some(reminder) = reminder {
        info!("⏲️🗑️ Reminder deleted: {reminder}");
        Ok(Some(format!("Reminder deleted: {reminder}")))
    } else {
        Ok(None)
    }
}

pub async fn notify(db: Surreal<Any>, matrix_client: Client) -> Result<()> {
    loop {
        sleep(TokioDuration::from_secs(1)).await;
        trace!("checking for due reminders");

        let due_reminders: Vec<Reminder> = db
            .query("select * from reminder where next_send_at and next_send_at < time::now()")
            .await?
            .take(0)?;

        if due_reminders.is_empty() {
            continue;
        }

        debug!("found {} due reminders", due_reminders.len());

        for r in due_reminders {
            let room_id = RoomId::parse(&r.room_id)?;
            let Some(room) = matrix_client.get_room(&room_id) else {
                warn!("room {room_id} not found to send reminder");
                continue;
            };

            // update reminder in db

            debug!("updating reminder {r} in db");
            let _r = db
                .query("fn::send_reminder($reminder)")
                .bind(("reminder", &r.id))
                .await?
                .check()
                .map_err(|err| {
                    warn!(
                        "fails to call fn::send_reminder in db on reminder {}, error: {err:?}",
                        r.id
                    )
                });

            // send reminder notification

            let content = RoomMessageEventContent::text_plain(format!("{}\n🔔🔔🔔", r.title));
            info!("🔔 sending reminder '{}' to room {room_id}", r.title);
            let _ = room
                .send(content)
                .await
                .map_err(|err| warn!("fails to send reminder to room {room_id}, error: {err:?}"));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Reminder {
    id: Thing,
    room_id: String,
    title: String,
    interval_unit: String,
    min_interval: usize,
    max_interval: Option<usize>,
    recurring: bool,
    last_sent_at: Option<DateTime<Utc>>,
    next_send_at: Option<DateTime<Utc>>,
}

impl Reminder {
    fn try_from_str(text: &str, room_id: &RoomId) -> Result<Self> {
        let re = Regex::new(
            r"!reminder([ ]*)(?P<recurring>(?i)[every ]*)(?P<min>[0-9]*)(?P<from_to>(?i)[^(0-9|m|h|d)]*)(?P<max>[0-9]*)(?P<delimiter>(?i)[^(0-9|m|h|d)]*)(?P<unit>(?i)[m|h|d]*)(.*):(?P<msg>.{1,200})",
        )?;

        let Some(groups) = re.captures_iter(text).next() else {
            bail!("Invalid reminder format")
        };

        let groups = groups
            .iter()
            .map(|g| g.unwrap().as_str())
            .collect::<Vec<&str>>();

        let recurring = groups.get(2).map(|s| !s.is_empty()).unwrap_or(false);
        let Some(min) = groups.get(3) else {
            bail!("reminder min is required")
        };
        debug!("min: {min}");
        let Some(interval_unit) = groups.get(7).map(|s| s.trim().to_lowercase()) else {
            bail!("reminder interval_unit is required")
        };
        let interval_unit = if interval_unit.contains("m") {
            "minute"
        } else if interval_unit.contains("h") {
            "hour"
        } else if interval_unit.contains("d") {
            "day"
        } else {
            bail!("reminder interval_unit is invalid")
        };
        debug!("interval_unit: {interval_unit}");

        let Some(title) = groups.get(9).map(|s| s.trim()) else {
            bail!("reminder title is required")
        };
        debug!("title: {title}");

        let min: usize = min.trim().parse()?;
        let max: Option<usize> = groups
            .get(5)
            .and_then(|s| {
                Some(s.trim()).filter(|s| !s.is_empty()).map(|s| {
                    let max = s.parse().unwrap_or_else(|err| {
                        warn!("fails to parse {s} to max, {err:?}");
                        min
                    });
                    match max.cmp(&min) {
                        Ordering::Greater => Some(max),
                        _ => None,
                    }
                })
            })
            .flatten();

        let reminder = Reminder {
            id: Thing::from(("reminder", Uuid::new_v4().to_string().as_str())),
            room_id: room_id.to_string(),
            title: title.to_string(),
            interval_unit: interval_unit.to_string(),
            min_interval: min,
            max_interval: max,
            recurring,
            last_sent_at: None,
            next_send_at: None,
        };

        debug!("new reminder: {reminder:?}");

        Ok(reminder)
    }
}

impl Display for Reminder {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let recurring = if self.recurring { "every " } else { "" };
        let range = if let Some(max) = self.max_interval {
            format!("{} - {} {}s", self.min_interval, max, self.interval_unit)
        } else {
            format!(
                "{} {}{}",
                self.min_interval,
                self.interval_unit,
                if self.min_interval == 1 { "" } else { "s" }
            )
        };

        write!(f, "{recurring}{range}: {}", self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test as test_log;

    #[test_log]
    pub fn reminder_from_str() {
        let messages = vec![
            "!reminder 1 minutes: Take out the trash.",
            "!reminder 12 minute: Take out the trash",
            "!reminder 31 M: Take out the trash",
            "!reminder 14m: Take out the trash",
            "!reminder every 1 – 3 day: Take out the trash",
            "!reminder 1 hour: Take out the trash",
            "!reminder 13 hours: Take out the trash",
            "!reminder 1h: Take out the trash",
            "!reminder every 1 h : Take out the trash",
            "!reminder 1 day: Take out the trash",
            "!reminder 13 days: Take out the trash",
            "!reminder 1d: Take out the trash",
            "!reminder1 d: Take out the trash",
            "!reminder every 1 - 3 minutes: Take out the trash.",
            "!reminder 12-4 minute: Take out the trash",
            "!reminder 31 to 9 m: Take out the trash",
            "!reminder 14 g 9m: Take out the trash",
            "!reminder 1 until 38 hour: Take out the trash",
            "!reminder 13-3 hours: Take out the trash",
            "!reminder 1 to 388h: Take out the trash",
            "!reminder 1-4 h: Take out the trash",
            "!reminder every 1 – 3 day: Take out the trash",
            "!reminder 13 - 3 days: Take out the trash",
            "!reminder every1 -asfas 📹 3d: Take out the trash",
            "!reminder 1 -- to 3 d: Take out the trash",
            "!reminder 10 minutes: Check the oven.",
            "!reminder 10 minutes: Check the oven.",
            "!reminder 2 hours: Laundry is done.",
            "!reminder 10 days: Mow the lawn.",
            "!reminder every 2h: Drink water.",
            "!reminder every 2-5 days: Clean the bathroom.",
            "!reminder every 30 to 60 days: Get a haircut.",
            "!reminder 10 minutes: Check the oven",
            "!reminder 2 hours: Laundry is done",
            "!reminder 10 days: Mow the lawn",
            "!reminder every 42 days: Get a haircut.",
            "!reminder 1-3d: Go to the gym",
            "!reminder 1-3d: Go to the gym every 1-3 days",
        ];

        let room_id =
            RoomId::parse("!WBGmhYXnxVfSYOoHua:matrix.com").expect("fails to parse room_id");
        for m in messages {
            debug!("parsing reminder from: {m}");
            Reminder::try_from_str(m, &room_id).expect("reminder");
        }
    }
}
