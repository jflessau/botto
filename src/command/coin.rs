use crate::prelude::*;
use rand::seq::SliceRandom;

pub fn flip() -> String {
    let answers = ["Heads", "Tails"];
    answers
        .choose(&mut rand::thread_rng())
        .unwrap_or({
            error!("fails to select answer");
            &"I have no idea."
        })
        .to_string()
}
