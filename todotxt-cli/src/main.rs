use todotxt::parser::parse;

fn main() {
    let output =
        parse("x (A) 2022-12-20 2022-13-21 Hello, World +project @home @work test:200").unwrap();

    println!("{:#?}", output);
}
