use dialoguer::{theme::ColorfulTheme, MultiSelect};

pub struct SelectMenu<'a> {
    options: &'a [&'a str],
    defaults: Vec<bool>,
    prompt: &'a str,
}

impl<'a> SelectMenu<'a> {
    pub fn new(options: &'a [&'a str], prompt: &'a str) -> Self {
        let defaults = vec![true; options.len()];
        Self {
            options,
            defaults,
            prompt,
        }
    }

    pub fn interact(&self) -> Vec<usize> {
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(self.prompt)
            .items(&self.options[..])
            .defaults(&self.defaults[..])
            .interact()
            .unwrap();

        if selections.is_empty() {
            println!("You did not select anything :(");
        }

        selections
    }
}
