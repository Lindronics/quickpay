# Quickpay

[![Rust test and lint](https://github.com/Lindronics/truelayer-quickpay/actions/workflows/general.yml/badge.svg?branch=main)](https://github.com/Lindronics/truelayer-quickpay/actions/workflows/general.yml)

A CLI-tool for creating and authorising payments with TrueLayer.

Currently, there aren't many viable use cases for this besides testing the different payment flows.

## Example usage

```bash
pay --name "Ben Eficiary" --iban "NL84INGB2266765221" eur 1
```

## Configuration

To use this tool, you will have to set

You can either set the environment variables

```bash
export QUICKPAY__CLIENT_ID=""
export QUICKPAY__CLIENT_SECRET=""
export QUICKPAY__CLIENT_KID=""
export QUICKPAY__CLIENT_PRIVATE_KEY=""
export QUICKPAY__REDIRECT_URI=""
```

or create a file `$HOME/.config/quickpay.toml` containing the configuration:

```toml
client_id=""
client_secret=""
client_kid=""
client_private_key=""
redirect_uri=""
```
