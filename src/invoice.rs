use std::str::FromStr;

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::GenerateArgs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub title: String,

    pub logo: String,
    pub from: String,
    pub to: String,
    pub date: String,
    pub due: String,

    pub items: Vec<String>,
    pub quantities: Vec<u64>,
    pub rates: Vec<f64>,

    pub tax: f64,
    pub discount: f64,
    pub currency: Currency,

    pub note: String,

    pub output: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
    INR,
    RUB,
    KRW,
    BRL,
    SGD,
}

impl Invoice {
    pub fn default() -> Self {
        let now = Local::now();

        Invoice {
            id: now.format("%Y%m%d").to_string(),
            title: "INVOICE".to_string(),

            logo: String::new(),
            from: "Project Folded, Inc.".to_string(),
            to: "Untitled Corporation, Inc.".to_string(),
            date: now.format("%b %d, %Y").to_string(),
            due: (now + chrono::Duration::days(14))
                .format("%b %d, %Y")
                .to_string(),

            items: vec!["Paper Cranes".to_string()],
            quantities: vec![2],
            rates: vec![25.0],

            tax: 0.0,
            discount: 0.0,
            currency: Currency::INR,
            note: String::new(),

            output: None
        }
    }
}

impl From<GenerateArgs> for Invoice {
    fn from(args: GenerateArgs) -> Self {
        let mut invoice = Invoice::default();

        if let Some(v) = args.id {
            invoice.id = v;
        }

        if let Some(v) = args.logo {
            invoice.logo = v;
        }

        if let Some(v) = args.from {
            invoice.from = v;
        }

        if let Some(v) = args.to {
            invoice.to = v;
        }

        if let Some(v) = args.date {
            invoice.date = v;
        }

        if let Some(v) = args.due {
            invoice.due = v;
        }

        if let Some(items) = args.items {
            if !items.is_empty() {
                invoice.items = items;
            }
        }

        if let Some(quantities) = args.quantities {
            if !quantities.is_empty() {
                invoice.quantities = quantities;
            }
        }

        if let Some(prices) = args.prices {
            if !prices.is_empty() {
                invoice.rates = prices;
            }
        }

        invoice.tax = args.tax;
        invoice.discount = args.discount;
        invoice.currency = args.currency;

        if let Some(v) = args.note {
            invoice.note = v;
        }

        invoice.output = args.output;

        invoice
    }
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::INR => "₹",
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CNY => "¥",
            Currency::RUB => "₽",
            Currency::KRW => "₩",
            Currency::BRL => "R$",
            Currency::SGD => "SGD$",
        }
    }
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "INR" | "₹" => Ok(Currency::INR),

            "USD" | "$" => Ok(Currency::USD),

            "EUR" | "€" => Ok(Currency::EUR),

            "GBP" | "£" => Ok(Currency::GBP),

            "JPY" => Ok(Currency::JPY),

            "CNY" => Ok(Currency::CNY),

            _ => Err(format!("Invalid currency: {}", s)),
        }
    }
}
