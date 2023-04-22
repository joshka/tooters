fn default_colour_map(annotation: &RichAnnotation) -> (String, String) {
    use RichAnnotation::*;
    match annotation {
        Default => ("".into(), "".into()),
        Link(link) => (format!("\x1B]8;;{link}\x1B\\"), format!("\x1B]8;;\x1B\\")),
        Image(_) => ("".into(), "".into()),
        Emphasis => (String::from("\x1B[3m"), String::from("\x1B[23m")),
        Strong => (String::from("\x1B[1m"), String::from("\x1B[22m")),
        Strikeout => (String::from("\x1B[9m"), String::from("\x1B[29m")),
        Code => (String::from("`"), String::from("`")),
        Preformat(_) => (String::from("```"), String::from("```")),
    }
}
