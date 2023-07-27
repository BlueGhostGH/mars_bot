#![feature(fs_try_exists)]

use mars_bot::logic::GameState;

fn main() {
    let id = {
        println!("Enter ID:");
        let mut id = String::new();
        ::std::io::stdin().read_line(&mut id).unwrap();

        id.trim().parse::<usize>().unwrap()
    };
    let mut round = 0usize;
    let mut state: Option<GameState> = None;

    loop {
        let path = format!("{}/s{id}_{round}.txt", ::std::env::args().nth(1).unwrap());
        let exists = std::fs::try_exists(&path).unwrap_or(false);

        if exists {
            ::std::thread::sleep(::std::time::Duration::from_millis(10));
        } else {
            continue;
        }

        let input = ::std::fs::read_to_string(path);

        if let Ok(input) = input {
            let input = mars_bot::game::input::GameInput::try_from(input.as_str()).unwrap();

            let mut new_state = GameState::process_input(state, input);

            let output = new_state.magic();
            state = Some(new_state);

            // dbg!(output);

            ::std::fs::write(
                format!("{}/c{id}_{round}.txt", ::std::env::args().nth(1).unwrap()),
                <mars_bot::game::output::GameOutput as Into<String>>::into(output),
            )
            .unwrap();

            round += 1;
        } else {
            continue;
        }
    }
}
