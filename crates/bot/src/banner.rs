use owo_colors::{AnsiColors, OwoColorize};
use rand::seq::SliceRandom;

const BANNER: &str = include_str!("../../../banner.txt");
const COLORS: [AnsiColors; 6] = [
    AnsiColors::Red,
    AnsiColors::Green,
    AnsiColors::Yellow,
    AnsiColors::Blue,
    AnsiColors::Magenta,
    AnsiColors::Cyan,
];

pub fn print() {
    let lines = BANNER.split('\n').collect::<Vec<_>>();

	for line in lines {
		let color = COLORS.choose(&mut rand::thread_rng()).unwrap();

		println!("{}", line.color(*color));
	}
}
