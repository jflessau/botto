pub fn text() -> String {
    r#"Here is a list of my commands:
!botto - this help message

!coinflip - flip a coin and get heads or tails
!nominate - get a random user from this chatroom

!r - roll the dice
e.g. !r d6, !r 2d8, !r 2d6 - 1d4 + 3
"#
    .to_string()
}
