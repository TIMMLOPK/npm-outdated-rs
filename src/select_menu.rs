use dialoguer::{theme::ColorfulTheme, MultiSelect};

pub fn select_menu(options: &[&str]) -> Vec<usize> {
    let defaults = vec![true; options.len()];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick the dependencies you want to update")
        .items(&options[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    if selections.is_empty() {
        println!("You did not select anything :(");
    }

    selections
}
