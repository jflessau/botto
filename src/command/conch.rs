use crate::prelude::*;
use rand::seq::SliceRandom;

pub fn answer() -> String {
    info!("🐚 answering");
    let answers = [
        "Maybe someday.",
        "Nothing.",
        "Neither.",
        "I don't think so.",
        "No.",
        "Yes.",
        "Try asking again.",
        "You cannot get to the top by sitting on your bottom.",
        "I see a new sauce in your future.",
        "Ask next time.",
        "Follow the seahorse.",
    ];

    answers
        .choose(&mut rand::thread_rng())
        .unwrap_or({
            error!("fails to select answer");
            &"I have no idea."
        })
        .to_string()
}
