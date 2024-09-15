# 🤖 botto

<img alt="chat bot's avatar image showing a robots shiny metal face" src="avatar-circle.png" width="180px"/>

A [matrix](https://matrix.org) chat bot written in rust using the [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk).

## Development

1. Rename `.example.env` to `.env` and enter your credentials
2. Start the [SurrealDB](https://surrealdb.com) database with Docker Compose
3. Run the bot with Cargo
4. (Optional) Run the tests

Here are the respective commands in order:

```bash
cp .example.env .env
docker compose up -d
cargo run
cargo test
```

## Usage

Add the bot to a room and type a message starting with `!botto` to get a list of its commands.

### Commands

#### ℹ️ Bot info

`!botto` -> `list of commands`

#### 🎲 Roll dice

`!r 1d20` -> `8`  
`!r 1d20 + 2d8 - 1d4 + 3` -> `15 + (8 + 4) - 4 + 3 🟰 26`

#### 🪙 Coin flip

`!coinflip` -> `Heads`

#### 🐚 Magic conch shell

`!conch` -> `Maybe someday.`

#### 👤 Random user from the chat

`!nominate` -> `Jane Doe`

#### 🔘 Choose from a list

`!choose pizza, pasta, sushi` -> `pizza`

#### 🪨 Rock, paper, scissors

`!rps rock` -> `🪨 💥 ✂️  -  You win!`

#### ⏲️ Reminder

Set reminders in x minutes, hours or days:

`!reminder 1 minute: Check the oven`  
`!reminder 2 hours: Feed the cat`  
`!reminder 10d: Go swimming`

- Recurring: `!remind every 2h: Drink water`
- With a random time: `!remind 1-3d: Go to the gym`
- Recurring with a random time: `!remind every 10 to 20 days: Do the thing`

The random time interval is recalculated each time the reminder is sent, meaning a 1-3 day reminder could trigger after `1.5` days the first time and `2.2` days the next.

**Hint:** Combine with other commands: `!reminder every 1-2d: !choose gym, run, swim`

##### Manage reminders:

`!reminders` -> list of reminders  
`!deletereminder 3` -> delete 3rd reminder from the list  
`!deleteAllReminders` -> delete all reminders
