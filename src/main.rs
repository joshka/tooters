use indoc::eprintdoc;

fn main() {
    eprintdoc! {"
        toot-rs was renamed to tooters. To install run:

            cargo install tooters --locked
        
        For more information, see https:://crates.io/crates/tooters or https://github.com/joshka/tooters
    "};
}
