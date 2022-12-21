use anyhow::anyhow;
use dialoguer::theme::ColorfulTheme;
use truelayer_rust::apis::payments::AdditionalInputOption;

pub fn text_input(
    display_text: &str,
    min_length: i32,
    max_length: i32,
    sensitive: bool,
) -> anyhow::Result<String> {
    let input = match sensitive {
        true => dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt(display_text)
            .interact()?,
        false => dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt(display_text)
            .validate_with(|s: &String| -> Result<(), String> {
                validate_text(s, min_length, max_length)
            })
            .interact()?,
    };
    Ok(input)
}

pub fn select_input(
    display_text: &str,
    options: &[AdditionalInputOption],
) -> anyhow::Result<String> {
    let option_keys: Vec<&str> = options
        .iter()
        .map(|x| x.display_text.default.as_ref())
        .collect();
    let index = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .items(&option_keys)
        .with_prompt(display_text)
        .interact()?;

    let selected_item = options.get(index).ok_or_else(|| anyhow!("asdf"))?;
    Ok(selected_item.id.clone())
}

fn validate_text(s: &str, min_length: i32, max_length: i32) -> Result<(), String> {
    let len = s.len() as i32;
    if len > max_length || len < min_length {
        Err(format!(
            "Should have length between {min_length} and {max_length}, was {len}"
        ))
    } else {
        Ok(())
    }
}
