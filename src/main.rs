use std::collections::HashMap;

use robinson::dom::{elem, text};

fn main() {
    let node = elem(
        String::from("div"),
        HashMap::new(),
        vec![
            elem(
                String::from("h1"),
                HashMap::new(),
                vec![text(String::from("h1 text"))],
            ),
            elem(
                String::from("h2"),
                HashMap::new(),
                vec![text(String::from("h2 text"))],
            ),
            elem(
                String::from("h3"),
                HashMap::new(),
                vec![text(String::from("h3 text"))],
            ),
        ],
    );

    println!("{}", node);
}
