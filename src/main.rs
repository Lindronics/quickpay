use clap::{Parser, ValueEnum};
use truelayer_quickpay::create;
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let identifier = if let Some(iban) = args.iban {
        AccountIdentifier::Iban { iban }
    } else if let Some(scan) = args.scan {
        AccountIdentifier::SortCodeAccountNumber {
            sort_code: scan.get(0).unwrap().to_string(),
            account_number: scan.get(1).unwrap().to_string(),
        }
    } else {
        panic!()
    };

    let beneficiary = Beneficiary::ExternalAccount {
        account_holder_name: args.name,
        reference: args.reference.unwrap_or_else(|| "reference".into()),
        account_identifier: identifier,
    };

    let credentials = Credentials::ClientCredentials {
        client_id: env!("TL_CLIENT_ID").into(),
        client_secret: env!("TL_CLIENT_SECRET").into(),
        scope: "payments".into(),
    };
    let client = TrueLayerClient::builder(credentials)
        .with_environment(Environment::Sandbox)
        .with_signing_key(env!("TL_CLIENT_KID"), env!("TL_CLIENT_PRIVATE_KEY").into())
        .build();

    create(&client, args.amount, args.currency_code.into(), beneficiary)
        .await
        .unwrap();
}
