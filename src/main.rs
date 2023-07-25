use mars_bot::GameOutput;

fn main()
{
    let id = {
        println!("Enter ID:");
        let mut id = String::new();
        ::std::io::stdin().read_line(&mut id).unwrap();

        id.trim().parse::<usize>().unwrap()
    };
    let mut round = 0usize;

    loop {
        let input = ::std::fs::read_to_string(format!(
            "{}/s{id}_{round}.txt",
            ::std::env::args().nth(1).unwrap()
        ));

        if let Ok(input) = input {
            let input = mars_bot::GameInput::try_from(input.as_str()).unwrap();

            // TODO: magic here

            let output = mars_bot::magic(input);

            ::std::fs::write(
                format!("{}/c{id}_{round}.txt", ::std::env::args().nth(1).unwrap()),
                <GameOutput as Into<String>>::into(output),
            )
            .unwrap();

            round += 1;
        } else {
            continue;
        }
    }
}
