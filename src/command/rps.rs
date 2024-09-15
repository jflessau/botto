use crate::prelude::*;
use rand::seq::SliceRandom;
use std::cmp::Ordering;

pub fn play(text: String) -> String {
    match Rps::try_from_string(text) {
        Ok(user_choice) => {
            let options = [Rps::Rock, Rps::Paper, Rps::Scissors];
            let bot_choice = options.choose(&mut rand::thread_rng()).unwrap_or_else(|| {
                warn!("fails to choose rps option");
                &Rps::Rock
            });

            let res = user_choice.defeats(bot_choice);

            format!(
                "{user_choice} 💥 {bot_choice}\n{}",
                match res {
                    Ordering::Less => "You lose.",
                    Ordering::Equal => "It's a tie.",
                    Ordering::Greater => "You win!",
                }
            )
        }
        Err(err) => err,
    }
}

#[derive(PartialEq, Eq)]
enum Rps {
    Rock,
    Paper,
    Scissors,
}

impl Rps {
    fn try_from_string(text: String) -> Result<Self, String> {
        match text.to_lowercase().trim() {
            "rock" => Ok(Self::Rock),
            "paper" => Ok(Self::Paper),
            "scissors" => Ok(Self::Scissors),
            _ => Err("Invalid choice. Please choose 'rock', 'paper', or 'scissors'.".to_string()),
        }
    }

    fn defeats(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (Self::Rock, Self::Scissors) => Ordering::Greater,
            (Self::Paper, Self::Rock) => Ordering::Greater,
            (Self::Scissors, Self::Paper) => Ordering::Greater,
            _ => Ordering::Less,
        }
    }
}

impl Display for Rps {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::Rock => "🪨",
                Self::Paper => "📄",
                Self::Scissors => "✂️",
            }
        )
    }
}
