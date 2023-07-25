fn main()
{
    let file = ::std::fs::read_to_string(::std::env::args().next().unwrap()).unwrap();
    let input = mars_bot::GameInput::try_from(file.as_str()).unwrap();

    dbg!(input);
}
