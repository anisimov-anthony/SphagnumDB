// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

mod sprout;
use crate::sprout::sprout::Sprout;

fn main() {

    let sp = Sprout::new();
    let name = sp.get_passport().get_name();
    let number = sp.get_data_storage().get_number();

    println!("Name: {}, Number: {}", name, number);
}
