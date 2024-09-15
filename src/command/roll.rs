use crate::prelude::*;
use rand::Rng;
use regex::Regex;

pub fn dice(text: &str) -> String {
    let re =
        Regex::new(r"(([-+]{0,1})([ ]*)([0-9]{0,10})([ dD]*)([0-9]+))").expect("regex is valid");

    let captures: Vec<String> = re
        .captures_iter(text)
        .flat_map(|caps| {
            caps.get(0)
                .map(|c| c.as_str().trim().to_lowercase().to_string())
        })
        .collect();

    debug!("captures: {captures:?}");

    let items = captures
        .iter()
        .flat_map(|capture| {
            let mut s = capture.clone();
            let positive = !capture.contains('-');

            s = s.replace("+", "");
            s = s.replace("-", "");
            s = s.replace(" ", "");

            let parts: Vec<&str> = s.split('d').collect();
            debug!("parts: {parts:?}");

            let item = match parts.len() {
                0 => Some(Item::Number {
                    positive,
                    number: 0,
                }),
                1 => Some(Item::Number {
                    positive,
                    number: parts.first().unwrap_or(&"0").parse().unwrap_or_default(),
                }),
                2 => parts
                    .first()
                    .map(|a| a.parse().unwrap_or(0))
                    .filter(|&a| a > 0)
                    .map(|amount| Item::Dices {
                        positive,
                        amount,
                        sides: parts.get(1).unwrap_or(&"1").parse().unwrap_or_default(),
                    }),
                _ => Some(Item::Number {
                    positive,
                    number: 0,
                }),
            };

            item
        })
        .collect::<Vec<_>>();

    let results = items
        .iter()
        .enumerate()
        .map(|(n, item)| {
            trace!("item: {:?} => {:?}", item, item.result(n == 0));
            item.result(n == 0)
        })
        .collect::<Vec<_>>();

    let sum = results.iter().map(|r| r.0).sum::<i64>();

    let explanation = results
        .iter()
        .map(|r| r.1.clone())
        .collect::<Vec<String>>()
        .join(" ");

    let dice_count = items.iter().map(|i| i.dice_count()).sum::<usize>();

    info!("🎲 rolling: {explanation} => {sum}");

    if dice_count > 1 {
        format!("{explanation}\n\n🟰 {sum}")
    } else {
        format!("{sum}")
    }
}

#[derive(Debug)]
pub enum Item {
    Number {
        positive: bool,
        number: usize,
    },
    Dices {
        positive: bool,
        amount: usize,
        sides: usize,
    },
}

impl Item {
    fn positive(&self) -> bool {
        match self {
            Item::Number { positive, .. } => *positive,
            Item::Dices { positive, .. } => *positive,
        }
    }

    fn result(&self, first: bool) -> (i64, String) {
        let result = match self {
            Item::Number { number, .. } => {
                let v = *number as i64;
                (v, format!("{v}"))
            }
            Item::Dices {
                mut amount, sides, ..
            } => {
                let mut rng = rand::thread_rng();
                let mut rolls = vec![];
                if amount == 0 {
                    amount = 1
                }

                let rolls_str = match amount {
                    0 => "".to_string(),
                    1 => {
                        if sides < &1 {
                            rolls.push(0);
                            "0".to_string()
                        } else {
                            let roll = rng.gen_range(1..=*sides);
                            rolls.push(roll);
                            format!("{roll}")
                        }
                    }
                    _ => {
                        for _ in 0..amount {
                            if sides < &1 {
                                rolls.push(0);
                            } else {
                                let roll = rng.gen_range(1..=*sides);
                                rolls.push(roll);
                            }
                        }

                        format!(
                            "({})",
                            rolls
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(" + ")
                        )
                    }
                };

                (rolls.iter().map(|r| *r as i64).sum::<i64>(), rolls_str)
            }
        };

        let s = match (self.positive(), first) {
            (true, true) => result.1,
            (true, false) => format!("+ {}", result.1),
            (false, false) => format!("- {}", result.1),
            (false, true) => format!("-{}", result.1),
        };

        let v = if self.positive() { result.0 } else { -result.0 };

        debug!("item: {:?}, {}, {}", self, v, s);

        (v, s)
    }

    fn dice_count(&self) -> usize {
        match self {
            Item::Number { .. } => 1,
            Item::Dices { amount, .. } => *amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn number_result() {
        let (v, s) = Item::Number {
            positive: true,
            number: 1,
        }
        .result(true);
        assert_eq!(v, 1);
        assert_eq!(s, "1");

        let (v, s) = Item::Number {
            positive: true,
            number: 1,
        }
        .result(false);
        assert_eq!(v, 1);
        assert_eq!(s, "+ 1");

        let (v, s) = Item::Number {
            positive: false,
            number: 1,
        }
        .result(true);
        assert_eq!(v, -1);
        assert_eq!(s, "-1");

        let (v, s) = Item::Number {
            positive: false,
            number: 1,
        }
        .result(false);
        assert_eq!(v, -1);
        assert_eq!(s, "- 1");
    }

    #[test]
    pub fn dice_result() {
        // first

        let (v, s) = Item::Dices {
            positive: true,
            amount: 0,
            sides: 0,
        }
        .result(true);
        assert_eq!(v, 0);
        assert_eq!(s, "0");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 1,
            sides: 0,
        }
        .result(true);
        assert_eq!(v, 0);
        assert_eq!(s, "0");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 1,
            sides: 0,
        }
        .result(true);
        assert_eq!(v, 0);
        assert_eq!(s, "-0");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 1,
            sides: 1,
        }
        .result(true);
        assert_eq!(v, 1);
        assert_eq!(s, "1");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 1,
            sides: 1,
        }
        .result(true);
        assert_eq!(v, -1);
        assert_eq!(s, "-1");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 2,
            sides: 1,
        }
        .result(true);
        assert_eq!(v, 2);
        assert_eq!(s, "(1 + 1)");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 2,
            sides: 1,
        }
        .result(true);
        assert_eq!(v, -2);
        assert_eq!(s, "-(1 + 1)");

        // not first

        let (v, s) = Item::Dices {
            positive: true,
            amount: 0,
            sides: 0,
        }
        .result(false);
        assert_eq!(v, 0);
        assert_eq!(s, "+ 0");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 1,
            sides: 0,
        }
        .result(false);
        assert_eq!(v, 0);
        assert_eq!(s, "+ 0");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 1,
            sides: 0,
        }
        .result(false);
        assert_eq!(v, 0);
        assert_eq!(s, "- 0");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 1,
            sides: 1,
        }
        .result(false);
        assert_eq!(v, 1);
        assert_eq!(s, "+ 1");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 1,
            sides: 1,
        }
        .result(false);
        assert_eq!(v, -1);
        assert_eq!(s, "- 1");

        let (v, s) = Item::Dices {
            positive: true,
            amount: 2,
            sides: 1,
        }
        .result(false);
        assert_eq!(v, 2);
        assert_eq!(s, "+ (1 + 1)");

        let (v, s) = Item::Dices {
            positive: false,
            amount: 2,
            sides: 1,
        }
        .result(false);
        assert_eq!(v, -2);
        assert_eq!(s, "- (1 + 1)");
    }
}
