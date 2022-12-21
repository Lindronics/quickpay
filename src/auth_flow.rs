use std::collections::HashMap;

use dialoguer::theme::ColorfulTheme;
use truelayer_rust::{
    apis::payments::{
        AdditionalInput, AdditionalInputImage, AdditionalInputType, AuthorizationFlow,
        AuthorizationFlowNextAction, ConsentSupported, CountryCode, FormSupported, Provider,
        ProviderSelectionSupported, RedirectSupported, StartAuthorizationFlowRequest,
        SubmitFormActionRequest, SubmitProviderSelectionActionRequest,
    },
    TrueLayerClient,
};

use crate::inputs::{select_input, text_input};

pub async fn handle_auth_flow(client: &TrueLayerClient, payment_id: &str) -> anyhow::Result<()> {
    let mut auth_flow = client
        .payments
        .start_authorization_flow(
            payment_id,
            &StartAuthorizationFlowRequest {
                provider_selection: Some(ProviderSelectionSupported {}),
                redirect: Some(RedirectSupported {
                    return_uri: "http://localhost:3000/callback".into(),
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
        match auth_flow_inner.actions.unwrap().next {
            AuthorizationFlowNextAction::ProviderSelection { providers } => {
                auth_flow = handle_provider_selection(client, payment_id, &providers).await;
            }
            AuthorizationFlowNextAction::Consent { .. } => {
                auth_flow = handle_consent_action(client, payment_id).await;
            }
            AuthorizationFlowNextAction::Redirect { uri, .. } => {
                println!("{uri}");
                auth_flow = None
            }
            AuthorizationFlowNextAction::Form { inputs, .. } => {
                auth_flow = handle_form_action(client, payment_id, &inputs).await;
            }
            AuthorizationFlowNextAction::Wait => auth_flow = None,
        }
    }
    Ok(())
}

async fn handle_provider_selection(
    client: &TrueLayerClient,
    payment_id: &str,
    providers: &[Provider],
) -> Option<AuthorizationFlow> {
    let provider_names = providers
        .iter()
        .flat_map(|p| {
            if let (Some(name), Some(country)) = (p.display_name.as_ref(), p.country_code.as_ref())
            {
                return Some(format!("{}  {}", to_emoji(country), name));
            }
            None
        })
        .collect::<Vec<_>>();
    let index = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .items(&provider_names)
        .with_prompt("Select provider")
        .interact()
        .unwrap();
    let selected_provider = &providers.get(index).unwrap().id;
    let response = client
        .payments
        .submit_provider_selection(
            payment_id,
            &SubmitProviderSelectionActionRequest {
                provider_id: selected_provider.clone(),
            },
        )
        .await
        .unwrap();
    response.authorization_flow
}

async fn handle_consent_action(
    client: &TrueLayerClient,
    payment_id: &str,
) -> Option<AuthorizationFlow> {
    let consent = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Submit consent")
        .wait_for_newline(false)
        .interact()
        .unwrap();
    if !consent {
        panic!("no consent")
    }
    let response = client.payments.submit_consent(payment_id).await.unwrap();
    response.authorization_flow
}

async fn handle_form_action(
    client: &TrueLayerClient,
    payment_id: &str,
    inputs: &[AdditionalInput],
) -> Option<AuthorizationFlow> {
    let mut submissions: HashMap<String, String> = HashMap::with_capacity(inputs.len());
    for input in inputs {
        match input {
            AdditionalInput::Text {
                id,
                display_text,
                sensitive,
                min_length,
                max_length,
                ..
            } => submissions.insert(
                id.to_string(),
                text_input(&display_text.default, *min_length, *max_length, *sensitive).unwrap(),
            ),
            AdditionalInput::TextWithImage {
                id,
                display_text,
                sensitive,
                min_length,
                max_length,
                image,
                ..
            } => {
                let img = match image {
                    AdditionalInputImage::Uri { .. } => panic!("URL images are not yet supported"),
                    AdditionalInputImage::Base64 { data, .. } => {
                        let img_bytes = base64::decode(data).unwrap();
                        image::load_from_memory(&img_bytes).unwrap()
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
                    text_input(&display_text.default, *min_length, *max_length, *sensitive)
                        .unwrap(),
                )
            }
            AdditionalInput::Select {
                id,
                display_text,
                options,
                ..
            } => submissions.insert(
                id.to_string(),
                select_input(&display_text.default, options).unwrap(),
            ),
        };
    }
    let response = client
        .payments
        .submit_form_inputs(
            payment_id,
            &SubmitFormActionRequest {
                inputs: submissions,
            },
        )
        .await
        .unwrap();
    response.authorization_flow
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
