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
            ::std::fs::write(
                format!("{}/c{id}_{round}.txt", ::std::env::args().nth(1).unwrap()),
                mars_bot::bot::Bot::turn(input),
            )
            .unwrap();

            round += 1;
        } else {
            continue;
        }

        ::std::thread::sleep(::std::time::Duration::from_secs(2));
    }
}
