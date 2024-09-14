use crate::prelude::*;

pub fn text() -> String {
    info!("ℹ️ helping");

    r#"🤖 Here is a list of my commands:

ℹ️ - help
!botto

🪙 - coin flip
!coinflip

👤 - get random user
!nominate

🎲 - dice roll
!r d6
!r 2d8
!r 2d6 - 1d4 + 3

⏲️ - reminder [minutes, hours, days]
!reminder 10 minutes: Check the oven
!reminder 2 hours: Laundry is done
!reminder 10 days: Mow the lawn

⏲️🔁 - recurring reminder
!reminder every 42 days: Get a haircut.

⏲️ 🔀 - random reminder
!reminder 1-3d: Go to the gym every 1-3 days

⏲️⚙️ - manage reminders
!reminders - list all reminders
!deleteReminder 3 - delete 3rd reminder from list
!deleteAllReminders - delete all reminders


🔗 Bot's source code: 
https://github.com/jflessau/botto
"#
    .to_string()
}
