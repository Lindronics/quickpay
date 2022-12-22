use anyhow::{anyhow, Context};
use dialoguer::theme::ColorfulTheme;
use regex::Regex;
use truelayer_rust::apis::payments::{AdditionalInputOption, AdditionalInputRegex};

pub fn text_input(
    display_text: &str,
    min_length: i32,
    max_length: i32,
    regexes: &[AdditionalInputRegex],
    sensitive: bool,
) -> Result<String, anyhow::Error> {
    let input = match sensitive {
        true => dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt(display_text)
            .interact()?,
        false => dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt(display_text)
            .validate_with(|s: &String| validate_text(s, min_length, max_length, regexes))
            .interact()?,
    };
    Ok(input)
}

pub fn select_input(
    display_text: &str,
    options: &[AdditionalInputOption],
) -> Result<String, anyhow::Error> {
    let option_keys: Vec<&str> = options
        .iter()
        .map(|x| x.display_text.default.as_ref())
        .collect();
    let index = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .items(&option_keys)
        .with_prompt(display_text)
        .interact()?;

    let selected_item = options.get(index).ok_or_else(|| anyhow!("bad index"))?;
    Ok(selected_item.id.clone())
}

fn validate_text(
    s: &str,
    min_length: i32,
    max_length: i32,
    regexes: &[AdditionalInputRegex],
) -> Result<(), anyhow::Error> {
    let len = s.len() as i32;
    anyhow::ensure!(
        len >= min_length || len <= max_length,
        "Should have length between {min_length} and {max_length}, was {len}"
    );
    for regex in regexes {
        let re = Regex::new(&regex.regex).context("could not build validation regex")?;
        anyhow::ensure!(re.is_match(s), regex.message.default.clone());
    }
    Ok(())
}
