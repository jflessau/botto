use crate::prelude::*;
use rand::seq::SliceRandom;

pub fn option(text: String) -> String {
    let options = text.split(',').map(str::trim).collect::<Vec<_>>();
    if options.len() < 2 {
        return "Please provide at least two options.".into();
    }
    let choice = options
        .choose(&mut rand::thread_rng())
        .unwrap_or(&"I have no idea.");

    info!("🎰 choosing '{}' from {:?}", choice, options);

    choice.to_string()
}
