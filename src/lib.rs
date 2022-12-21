use anyhow::bail;
use auth_flow::handle_auth_flow;
use std::time::Duration;
use truelayer_rust::{
    apis::payments::{
        Beneficiary, CreatePaymentRequest, CreatePaymentUserRequest, Currency,
        PaymentMethodRequest, PaymentStatus, ProviderSelectionRequest, SchemeSelection,
    },
    pollable::PollOptions,
    PollableUntilTerminalState, TrueLayerClient,
};

mod auth_flow;
mod inputs;

pub async fn create(
    client: &TrueLayerClient,
    amount: u64,
    currency: Currency,
    beneficiary: Beneficiary,
) -> anyhow::Result<()> {
    let payment = client
        .payments
        .create(&CreatePaymentRequest {
            amount_in_minor: amount,
            currency,
            payment_method: PaymentMethodRequest::BankTransfer {
                provider_selection: ProviderSelectionRequest::UserSelected {
                    filter: None,
                    scheme_selection: Some(SchemeSelection::InstantPreferred {
                        allow_remitter_fee: Some(false),
                    }),
                },
                beneficiary,
            },
            user: CreatePaymentUserRequest::NewUser {
                name: Some("Name".into()),
                email: Some("a@b.com".into()),
                phone: None,
            },
            metadata: None,
        })
        .await?;

    handle_auth_flow(client, &payment.id).await?;

    let pb =
        indicatif::ProgressBar::new_spinner().with_message("Polling for terminal payment status");
    pb.enable_steady_tick(Duration::from_millis(100));
    let output = payment
        .poll_until_terminal_state(client, PollOptions::default())
        .await?;
    pb.finish_with_message(match output.status {
        PaymentStatus::Executed { executed_at, .. } => format!("Payment executed at {executed_at}"),
        PaymentStatus::Settled { settled_at, .. } => format!("Payment settled at {settled_at}"),
        PaymentStatus::Failed { failed_at, .. } => format!("Payment failed at {failed_at}"),
        _ => bail!("Payment did not reach terminal status"),
    });

    Ok(())
}
