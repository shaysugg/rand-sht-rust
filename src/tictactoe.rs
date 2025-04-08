use std::io::Write;

#[derive(PartialEq)]
enum GameState {
    Finished(Option<GameCommandMark>),
    Playing,
    New,
    Terminated,
}
#[derive(Debug)]
enum GameCommand {
    Mark(GameCommandMark, u8, u8),
    Quit,
    Restart,
}
#[derive(Debug)]
enum GameCommandMark {
    X,
    O,
}

type Table = Vec<Vec<String>>;

fn new_table() -> Table {
    vec![vec![String::from("-"); 3]; 3]
}

impl GameCommand {
    fn buid(string: &str) -> Result<GameCommand, &str> {
        fn parse_mark_command(string: &str) -> Option<GameCommand> {
            if string.len() != 3 {
                return None;
            }
            let mut chars = string.chars();
            let x = chars.next()?.to_digit(10)? as u8;
            let y = chars.next()?.to_digit(10)? as u8;
            if !(1..=3).contains(&x) || !(1..=3).contains(&y) {
                return None;
            }
            match GameCommandMark::from(chars.next().unwrap()) {
                Some(mark) => return Some(GameCommand::Mark(mark, x, y)),
                None => return None,
            }
        }

        match string.trim().to_lowercase().as_str() {
            "quit" => Ok(GameCommand::Quit),
            "restart" => Ok(GameCommand::Restart),
            _ => match parse_mark_command(string) {
                Some(command) => Ok(command),
                None => Err("Invalid command"),
            },
        }
    }
}

impl PartialEq for GameCommand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GameCommand::Quit, GameCommand::Quit) => true,
            (GameCommand::Restart, GameCommand::Restart) => true,
            (GameCommand::Mark(mark1, x1, y1), GameCommand::Mark(mark2, x2, y2)) => {
                mark1 == mark2 && x1 == x2 && y1 == y2
            }
            (_, _) => false,
        }
    }
}

impl GameCommandMark {
    fn from(char: char) -> Option<GameCommandMark> {
        match char.to_ascii_lowercase() {
            'x' => Some(GameCommandMark::X),
            'o' => Some(GameCommandMark::O),
            _ => None,
        }
    }

    fn to_char(&self) -> char {
        match self {
            GameCommandMark::O => 'O',
            GameCommandMark::X => 'X',
        }
    }

    fn toggle(&self) -> GameCommandMark {
        match self {
            GameCommandMark::O => GameCommandMark::X,
            GameCommandMark::X => GameCommandMark::O,
        }
    }
}

impl PartialEq for GameCommandMark {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GameCommandMark::X, GameCommandMark::X) => true,
            (GameCommandMark::O, GameCommandMark::O) => true,
            _ => false,
        }
    }
}

pub fn run_tictactoe() {
    let mut table = new_table();
    let mut game_state = GameState::New;
    let mut player_mark = GameCommandMark::X;

    while game_state != GameState::Terminated {
        match game_state {
            GameState::Playing => {
                let input = read_input(&player_mark);

                // Check if positon, add the mark of player
                let input = if input.len() == 2 {
                    input + player_mark.to_char().to_string().as_str()
                } else {
                    input
                };
                let command = GameCommand::buid(&input).unwrap();

                match command {
                    GameCommand::Mark(mark, x, y) => {
                        match GameCommandMark::from(
                            table[(x - 1) as usize][(y - 1) as usize]
                                .chars()
                                .next()
                                .unwrap(),
                        ) {
                            Some(_) => {
                                println!("Already marked");
                            }
                            None => {
                                table[(x - 1) as usize][(y - 1) as usize] =
                                    mark.to_char().to_string();
                                player_mark = player_mark.toggle();
                            }
                        }
                    }
                    GameCommand::Restart => {
                        game_state = GameState::New;
                    }
                    GameCommand::Quit => {
                        game_state = GameState::Terminated;
                    }
                }
                match is_somebody_won(&table) {
                    Some(mark) => {
                        game_state = GameState::Finished(Some(mark));
                    }
                    None => (),
                }
            }
            GameState::New => {
                println!("*** NEW GAME ***");
                println!(
                    r#"
Enter a mark,
for quiting enter quit,
for restarting enter restart.
                "#
                );
                table = new_table();
                game_state = GameState::Playing
            }
            GameState::Finished(ref winner) => {
                match winner {
                    Some(winner) => {
                        println!("----------------------------");
                        println!(
                            ">>> PLAYER {} WON THE GAME! <<<",
                            winner.to_char().to_string()
                        );
                        game_state = GameState::New;
                        println!("----------------------------");
                    }
                    None => println!(">>> Nobody wins :O <<<"),
                };
            }
            GameState::Terminated => return,
        }
        display_tictactoe(&table);
    }
}

fn display_tictactoe(table: &Table) {
    for row in table {
        for col in row {
            print!("{col}\t")
        }
        print!("\n")
    }
}

fn read_input(player_mark: &GameCommandMark) -> String {
    println!(
        "Player {} Enter the input! (format: xy, like: 21)",
        player_mark.to_char().to_string()
    );
    let mut input = String::new();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

fn is_somebody_won(table: &Table) -> Option<GameCommandMark> {
    fn equal3(str1: &str, str2: &str, str3: &str) -> Option<GameCommandMark> {
        match (
            str1.to_lowercase().as_str(),
            str2.to_lowercase().as_str(),
            str3.to_lowercase().as_str(),
        ) {
            ("x", "x", "x") => return Some(GameCommandMark::X),
            ("o", "o", "o") => return Some(GameCommandMark::O),
            _ => return None,
        }
    }

    for row in table {
        match equal3(&row[0], &row[1], &row[2]) {
            Some(mark) => return Some(mark),
            None => continue,
        }
    }

    for i in 0..3 {
        match equal3(&table[0][i], &table[1][i], &table[2][i]) {
            Some(mark) => return Some(mark),
            None => continue,
        }
    }

    match (
        equal3(&table[0][0], &table[1][1], &table[2][2]),
        equal3(&table[2][0], &table[1][1], &table[0][2]),
    ) {
        (_, Some(mark)) => return Some(mark),
        (Some(mark), _) => return Some(mark),
        _ => (),
    }

    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_command_from_string() {
        assert_eq!(GameCommand::buid("quit"), Ok(GameCommand::Quit));
        assert_eq!(GameCommand::buid("restart"), Ok(GameCommand::Restart));
        assert_eq!(
            GameCommand::buid("12X"),
            Ok(GameCommand::Mark(GameCommandMark::X, 1, 2))
        );
        assert_eq!(
            GameCommand::buid("13o"),
            Ok(GameCommand::Mark(GameCommandMark::O, 1, 3))
        );
        assert_eq!(GameCommand::buid("04o"), Err("Invalid command"));
    }

    #[test]
    fn test_is_somebody_won() {
        let table = new_table();
        assert_eq!(is_somebody_won(&table), None);

        let mut table = new_table();
        table[0][0] = String::from("x");
        table[0][1] = String::from("x");
        table[0][2] = String::from("x");

        assert_eq!(is_somebody_won(&table), Some(GameCommandMark::X));

        let mut table = new_table();
        table[0][0] = String::from("o");
        table[1][1] = String::from("o");
        table[2][2] = String::from("o");

        assert_eq!(is_somebody_won(&table), Some(GameCommandMark::O));

        let mut table = new_table();
        table[0][0] = String::from("o");
        table[1][0] = String::from("o");
        table[2][0] = String::from("o");

        assert_eq!(is_somebody_won(&table), Some(GameCommandMark::O));
    }
}
