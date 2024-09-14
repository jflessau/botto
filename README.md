🤖 botto

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

#### Bot info

`!botto` -> `list of commands`

#### Roll dice

`!r 1d20` -> `8`
`!r 1d20 + 2d8 - 1d4 + 3` -> `15 + (8 + 4) - 4 + 3 🟰 26`

#### Randomness

| cmd         | description                                                | example response |
| ----------- | ---------------------------------------------------------- | ---------------- |
| `!coinflip` | get a random coin side                                     | `Heads`          |
| `!conch`    | get a random answer from the magic conch shell (SpongeBob) | `Maybe someday.` |
| `!nominate` | get a random user from the chat                            | `Jane Doe`       |

#### Reminder

Set reminders in x minutes, hours or days:

`!reminder 1 minute: Check the oven`
`!reminder 2 hours: Feed the cat`
`!reminder 10d: Go swimming`

Set recurring reminder:
`!remind every 2h: Drink water`

Set reminder with a random time:
`!remind 1-3d: Go to the gym`

Set recurring reminder with a random time:
`!remind every 10 to 20 days: Do the thing`

The random time interval is recalculated each time the reminder is sent, meaning a 1-3 day reminder could trigger after `1.5` days the first time and `2.2` days the next.
