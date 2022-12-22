use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use truelayer_quickpay::{config::Configuration, QuickPayClient};
use truelayer_rust::{
    apis::{
        auth::Credentials,
        payments::{AccountIdentifier, Beneficiary},
    },
    client::Environment,
    TrueLayerClient,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Payment currency
    #[arg(value_enum)]
    currency_code: Currency,

    /// Payment amount in currency minor
    amount: u64,

    /// Sort code and account number, e.g. "010102,12345678"
    #[arg(short, long)]
    scan: Option<Vec<String>>,

    /// IBAN
    #[arg(short, long)]
    iban: Option<String>,

    /// Name of the beneficiary
    #[arg(short, long)]
    name: String,

    /// Payment reference
    #[arg(short, long)]
    reference: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Currency {
    /// GBP pence
    Gbp,
    /// EUR cent
    Eur,
}

impl From<Currency> for truelayer_rust::apis::payments::Currency {
    fn from(c: Currency) -> Self {
        match c {
            Currency::Gbp => Self::Gbp,
            Currency::Eur => Self::Eur,
        }
    }
}

fn account_identifier(
    scan: Option<Vec<String>>,
    iban: Option<String>,
) -> Result<AccountIdentifier, anyhow::Error> {
    if let Some(iban) = iban {
        return Ok(AccountIdentifier::Iban { iban });
    }
    if let Some(scan) = scan {
        if let (Some(sort_code), Some(account)) = (scan.get(0), scan.get(1)) {
            return Ok(AccountIdentifier::SortCodeAccountNumber {
                sort_code: sort_code.clone(),
                account_number: account.clone(),
            });
        }
    }
    Err(anyhow::anyhow!("mising account identifier"))
}

pub fn get_configuration() -> Result<Configuration, anyhow::Error> {
    let config = config::Config::builder()
        .add_source(
            config::File::from(
                dirs::home_dir()
                    .ok_or_else(|| anyhow!("could not find home dir"))?
                    .join(".config")
                    .join("quickpay"),
            )
            .required(true),
        )
        .add_source(config::Environment::with_prefix("QUICKPAY").separator("__"))
        .build()?
        .try_deserialize()?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let beneficiary = Beneficiary::ExternalAccount {
        account_holder_name: args.name,
        reference: args.reference.unwrap_or_else(|| "reference".into()),
        account_identifier: account_identifier(args.scan, args.iban)?,
    };

    let configuration = get_configuration()?;

    let client = QuickPayClient {
        tl: TrueLayerClient::builder(Credentials::ClientCredentials {
            client_id: configuration.client_id,
            client_secret: configuration.client_secret.into(),
            scope: "payments".into(),
        })
        .with_environment(Environment::Sandbox)
        .with_signing_key(
            &configuration.client_kid,
            configuration.client_private_key.into_bytes(),
        )
        .build(),
        redirect_uri: configuration.redirect_uri,
    };

    client
        .create(args.amount, args.currency_code.into(), beneficiary)
        .await
}
