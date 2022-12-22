use std::collections::HashMap;

use dialoguer::theme::ColorfulTheme;
use truelayer_rust::apis::payments::{
    AdditionalInput, AdditionalInputImage, AdditionalInputType, AuthorizationFlow,
    AuthorizationFlowNextAction, ConsentSupported, CountryCode, FormSupported, Provider,
    ProviderSelectionSupported, RedirectSupported, StartAuthorizationFlowRequest,
    SubmitFormActionRequest, SubmitProviderSelectionActionRequest,
};

use crate::{
    inputs::{select_input, text_input},
    QuickPayClient,
};

impl QuickPayClient {
    pub async fn handle_auth_flow(&self, payment_id: &str) -> Result<(), anyhow::Error> {
        let mut auth_flow = self
            .tl
            .payments
            .start_authorization_flow(
                payment_id,
                &StartAuthorizationFlowRequest {
                    provider_selection: Some(ProviderSelectionSupported {}),
                    redirect: Some(RedirectSupported {
                        return_uri: self.redirect_uri.clone(),
                        direct_return_uri: None,
                    }),
                    consent: Some(ConsentSupported {}),
                    form: Some(FormSupported {
                        input_types: vec![
                            AdditionalInputType::Text,
                            AdditionalInputType::TextWithImage,
                            AdditionalInputType::Select,
                        ],
                    }),
                },
            )
            .await?
            .authorization_flow;

        while let Some(auth_flow_inner) = auth_flow {
            auth_flow = match auth_flow_inner
                .actions
                .ok_or_else(|| anyhow::anyhow!("no auth flow actions"))?
                .next
            {
                AuthorizationFlowNextAction::ProviderSelection { providers } => {
                    self.handle_provider_selection(payment_id, &providers)
                        .await?
                }
                AuthorizationFlowNextAction::Consent { .. } => {
                    self.handle_consent_action(payment_id).await?
                }
                AuthorizationFlowNextAction::Redirect { uri, .. } => {
                    self.handle_redirect_action(&uri)
                }
                AuthorizationFlowNextAction::Form { inputs, .. } => {
                    self.handle_form_action(payment_id, &inputs).await?
                }
                AuthorizationFlowNextAction::Wait => None,
            }
        }
        Ok(())
    }

    async fn handle_provider_selection(
        &self,
        payment_id: &str,
        providers: &[Provider],
    ) -> Result<Option<AuthorizationFlow>, anyhow::Error> {
        let provider_names = providers
            .iter()
            .flat_map(|p| {
                Some(format!(
                    "{}  {}",
                    to_emoji(p.country_code.as_ref()?),
                    p.display_name.as_ref()?
                ))
            })
            .collect::<Vec<_>>();
        let index = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .items(&provider_names)
            .with_prompt("Select provider")
            .interact()?;
        let selected_provider = &providers.get(index).expect("index out of bounds").id;
        let response = self
            .tl
            .payments
            .submit_provider_selection(
                payment_id,
                &SubmitProviderSelectionActionRequest {
                    provider_id: selected_provider.clone(),
                },
            )
            .await?;
        Ok(response.authorization_flow)
    }

    async fn handle_consent_action(
        &self,
        payment_id: &str,
    ) -> Result<Option<AuthorizationFlow>, anyhow::Error> {
        let consent = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Submit consent")
            .wait_for_newline(false)
            .interact()?;
        anyhow::ensure!(consent, "consent was not given");
        let response = self.tl.payments.submit_consent(payment_id).await?;
        Ok(response.authorization_flow)
    }

    async fn handle_form_action(
        &self,
        payment_id: &str,
        inputs: &[AdditionalInput],
    ) -> Result<Option<AuthorizationFlow>, anyhow::Error> {
        let mut submissions: HashMap<String, String> = HashMap::with_capacity(inputs.len());
        for input in inputs {
            match input {
                AdditionalInput::Text {
                    id,
                    display_text,
                    sensitive,
                    min_length,
                    max_length,
                    regexes,
                    ..
                } => submissions.insert(
                    id.to_string(),
                    text_input(
                        &display_text.default,
                        *min_length,
                        *max_length,
                        regexes,
                        *sensitive,
                    )?,
                ),
                AdditionalInput::TextWithImage {
                    id,
                    display_text,
                    sensitive,
                    min_length,
                    max_length,
                    regexes,
                    image,
                    ..
                } => {
                    let img = match image {
                        AdditionalInputImage::Uri { .. } => {
                            todo!("URL images are not yet supported")
                        }
                        AdditionalInputImage::Base64 { data, .. } => {
                            let img_bytes = base64::decode(data)?;
                            image::load_from_memory(&img_bytes)?
                        }
                    };
                    let conf = viuer::Config {
                        absolute_offset: false,
                        width: Some(64),
                        height: Some(20),
                        ..Default::default()
                    };
                    viuer::print(&img, &conf).expect("Image printing failed");
                    submissions.insert(
                        id.to_string(),
                        text_input(
                            &display_text.default,
                            *min_length,
                            *max_length,
                            regexes,
                            *sensitive,
                        )?,
                    )
                }
                AdditionalInput::Select {
                    id,
                    display_text,
                    options,
                    ..
                } => submissions.insert(
                    id.to_string(),
                    select_input(&display_text.default, options)?,
                ),
            };
        }
        let response = self
            .tl
            .payments
            .submit_form_inputs(
                payment_id,
                &SubmitFormActionRequest {
                    inputs: submissions,
                },
            )
            .await?;
        Ok(response.authorization_flow)
    }

    pub fn handle_redirect_action(&self, redirect_uri: &str) -> Option<AuthorizationFlow> {
        println!("Authorisation link: \n{redirect_uri}\n");
        None
    }
}

fn to_emoji(country: &CountryCode) -> &str {
    match country {
        CountryCode::DE => "🇩🇪",
        CountryCode::ES => "🇪🇸",
        CountryCode::FR => "🇫🇷",
        CountryCode::GB => "🇬🇧",
        CountryCode::IE => "🇮🇪",
        CountryCode::IT => "🇮🇹",
        CountryCode::LT => "🇱🇹",
        CountryCode::NL => "🇳🇱",
        CountryCode::PL => "🇵🇱",
        CountryCode::PT => "🇵🇹",
    }
}
