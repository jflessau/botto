use crate::prelude::*;
use regex::Regex;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub async fn new(room_id: &RoomId, text: &str, db: &Surreal<Any>) -> Result<String> {
    let reminder = Reminder::try_from_str(text)?;

    let res: Vec<String> = db
        .create(format!(
            r#"
            fn::create_reminder(
                "{room_id}",
                "{msg}", 
                "{time_unit}",
                {min},
                {max},
                {recurring}
            );
        "#,
            room_id = room_id,
            msg = reminder.message,
            time_unit = reminder.time_unit.to_surreal_str(),
            min = reminder.interval_min,
            max = reminder.interval_max,
            recurring = reminder.recurring,
        ))
        .await?;

    debug!("reminder created: {res:?}");

    Ok(reminder.to_string())
}

struct Reminder {
    interval_min: usize,
    interval_max: usize,
    time_unit: TimeUnit,
    recurring: bool,
    message: String,
}

impl Reminder {
    fn try_from_str(text: &str) -> Result<Self> {
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
        let Some(time_unit) = groups.get(7).map(|s| s.trim().to_lowercase()) else {
            bail!("reminder time_unit is required")
        };
        debug!("time_unit: {time_unit}");

        let Some(message) = groups.get(9) else {
            bail!("reminder message is required")
        };
        debug!("message: {message}");

        let time_unit = TimeUnit::try_from_str(time_unit)?;
        let min: usize = min.trim().parse()?;
        let mut max = groups
            .get(5)
            .and_then(|s| {
                Some(s.trim()).filter(|s| !s.is_empty()).map(|s| {
                    s.parse().unwrap_or_else(|err| {
                        warn!("fails to parse {s} to max, {err:?}");
                        min
                    })
                })
            })
            .unwrap_or(min);
        if max < min {
            max = min;
        }

        debug!("new reminder with: {min} - {max} {time_unit}: {message}");

        Ok(Reminder {
            interval_min: min,
            interval_max: max,
            time_unit,
            recurring,
            message: message.to_string(),
        })
    }
}

impl Display for Reminder {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let recurring = if self.recurring { "every " } else { "" };
        let range = if self.interval_min == self.interval_max {
            format!(
                "{} {}{}",
                self.interval_min,
                self.time_unit,
                if self.interval_min == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} - {} {}s",
                self.interval_min, self.interval_max, self.time_unit
            )
        };

        write!(f, "Reminder {recurring}{range}: {}", self.message)
    }
}

enum TimeUnit {
    Minute,
    Hour,
    Day,
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Minute => write!(f, "minute"),
            Self::Hour => write!(f, "hour"),
            Self::Day => write!(f, "day"),
        }
    }
}

impl TimeUnit {
    fn try_from_str(unit: String) -> Result<Self> {
        if unit.contains("m") {
            Ok(Self::Minute)
        } else if unit.contains("h") {
            Ok(Self::Hour)
        } else if unit.contains("d") {
            Ok(Self::Day)
        } else {
            bail!("Invalid time unit: {unit}")
        }
    }

    fn to_surreal_str(&self) -> String {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test as test_log;

    #[test_log]
    pub fn reminder_from_str() {
        let input = vec![
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
        ];

        for s in input {
            debug!("parsing reminder from: {s}");
            Reminder::try_from_str(s).expect("reminder");
        }
    }
}
