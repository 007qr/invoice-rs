mod invoice;

use std::fs;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use invoice::Invoice;
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::{Font, TextDirection};

use crate::invoice::Currency;

#[derive(Parser, Debug)]
#[command(name = "invoice")]
#[command(about = "Invoice generates invoices from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Generate(GenerateArgs),
}

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[arg(long)]
    id: Option<String>,

    #[arg(long, default_value = "INVOICE")]
    title: String,

    #[arg(long)]
    logo: Option<String>,

    #[arg(long)]
    from: Option<String>,

    #[arg(long)]
    to: Option<String>,

    #[arg(long)]
    date: Option<String>,

    #[arg(long)]
    due: Option<String>,

    #[arg(long)]
    items: Option<Vec<String>>,

    #[arg(long)]
    quantities: Option<Vec<u64>>,

    #[arg(long)]
    prices: Option<Vec<f64>>,

    #[arg(long, default_value_t = 0.0)]
    tax: f64,

    #[arg(long, default_value_t = 0.0)]
    discount: f64,

    #[arg(long, default_value = "INR")]
    currency: Currency,

    #[arg(long)]
    note: Option<String>,

    #[arg(long)]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate(args) => handle_generate(args),
    }
}

fn fill(r: u8, g: u8, b: u8) -> Fill {
    Fill {
        paint: rgb::Color::new(r, g, b).into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    }
}

fn handle_generate(args: GenerateArgs) {
    let invoice: Invoice = args.into();

    // A4 in points (1 pt = 1/72 inch).
    let page_w = 595.276_f32;
    let page_h = 841.89_f32;
    let margin = 40.0_f32;

    // Fonts: load the original Inter TTFs directly. krilla handles subsetting
    // and FontDescriptor scaling correctly, so no preprocessing needed.
    let regular = Font::new(
        Arc::new(fs::read("./Inter/Inter Hinted for Windows/Desktop/Inter-Regular.ttf").unwrap())
            .into(),
        0,
    )
    .expect("load Inter-Regular");
    let bold = Font::new(
        Arc::new(fs::read("./Inter/Inter Hinted for Windows/Desktop/Inter-Bold.ttf").unwrap())
            .into(),
        0,
    )
    .expect("load Inter-Bold");

    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::from_wh(page_w, page_h).unwrap());
    let mut surface = page.surface();

    // krilla uses top-down y coords, with text positioned at the baseline.
    // Track a cursor that walks down the page.
    let mut y = margin;

    // 1. "From" line — 12pt, dark grey.
    y += 12.0; // baseline drop for 12pt
    surface.set_fill(Some(fill(55, 55, 55)));
    surface.draw_text(
        Point::from_xy(margin, y),
        regular.clone(),
        13.0,
        &invoice.from,
        false,
        TextDirection::Auto,
    );

    // 2. Horizontal rule — half content width, light grey.
    y += 30.0;
    let content_w = page_w - 2.0 * margin;
    let half_w = content_w / 2.0;
    let rule = {
        let mut pb = PathBuilder::new();
        pb.move_to(margin, y);
        pb.line_to(margin + half_w, y);
        pb.finish().unwrap()
    };
    surface.set_fill(None);
    surface.set_stroke(Some(Stroke {
        paint: rgb::Color::new(225, 225, 225).into(),
        width: 1.2,
        ..Default::default()
    }));
    surface.draw_path(&rule);

    // 3. Title — 24pt bold, black.
    y += 6.0 + 60.0; // gap + baseline drop for 24pt
    surface.set_stroke(None);
    surface.set_fill(Some(fill(0, 0, 0)));
    surface.draw_text(
        Point::from_xy(margin, y),
        bold.clone(),
        24.0,
        &invoice.title,
        false,
        TextDirection::Auto,
    );

    // 4. Invoice number + Date — 12.5pt
    y += 6.0 + 20.0; // gap + baseline drop for 24pt
    surface.set_stroke(None);
    surface.set_fill(Some(fill(100, 100, 100)));
    surface.draw_text(
        Point::from_xy(margin, y),
        regular.clone(),
        12.5,
        &format!("#{}  ·  {}", invoice.id, invoice.date),
        false,
        TextDirection::Auto,
    );

    // 5. Bill to
    y += 6.0 + 42.0; // gap + baseline drop for 24pt
    surface.set_stroke(None);
    surface.set_fill(Some(fill(100, 100, 100)));
    surface.draw_text(
        Point::from_xy(margin, y),
        regular.clone(),
        10.0,
        "BILL TO",
        false,
        TextDirection::Auto,
    );

    // 6. Bill to Name
    y += 6.0 + 20.0; // gap + baseline drop for 24pt
    surface.set_stroke(None);
    surface.set_fill(Some(fill(55, 55, 55)));
    surface.draw_text(
        Point::from_xy(margin, y),
        regular.clone(),
        16.0,
        &invoice.to,
        false,
        TextDirection::Auto,
    );

    // 7. Items table — borderless, four columns. ITEM is wide; QTY/RATE/AMOUNT
    //    are pushed to the right via fixed x-offsets.
    let col_item = margin;
    let col_qty = 360.0;
    let col_rate = 405.0;
    let col_amount = 480.0;

    // Header row.
    y += 80.0;
    surface.set_fill(Some(fill(55, 55, 55)));
    for (x, label) in [
        (col_item, "ITEM"),
        (col_qty, "QTY"),
        (col_rate, "RATE"),
        (col_amount, "AMOUNT"),
    ] {
        surface.draw_text(
            Point::from_xy(x, y),
            regular.clone(),
            9.0,
            label,
            false,
            TextDirection::Auto,
        );
    }

    // Item rows.
    let sym = invoice.currency.symbol();
    surface.set_fill(Some(fill(0, 0, 0)));
    let mut subtotal = 0.0_f64;
    for i in 0..invoice.items.len() {
        y += 24.0;
        let item = invoice.items[i].clone();
        let qty = invoice.quantities.get(i).copied().unwrap_or(1);
        let rate = invoice.rates.get(i).copied().unwrap_or(0.0);
        let amount = qty as f64 * rate;
        subtotal += amount;

        let cells = [
            (col_item, item),
            (col_qty, qty.to_string()),
            (col_rate, format!("{}{:.2}", sym, rate)),
            (col_amount, format!("{}{:.2}", sym, amount)),
        ];
        for (x, text) in &cells {
            surface.draw_text(
                Point::from_xy(*x, y),
                regular.clone(),
                11.0,
                text,
                false,
                TextDirection::Auto,
            );
        }
    }

    let total = subtotal + invoice.tax - invoice.discount;

    let mut rows = vec![
        ("Subtotal", format!("{}{:.2}", sym, subtotal)),
        ("Total", format!("{}{:.2}", sym, total)),
        ("Due Date", invoice.due.clone()),
    ];

    if invoice.tax > 0.0 {
        rows.insert(1, ("Tax", format!("{:.2}", invoice.tax)))
    }

    if invoice.discount > 0.0 {
        rows.insert(rows.len() - 2, ("Discount", format!("{:.2}", invoice.discount)))
    }

    y += 240.0;
    for (label, value) in rows {
        y += 24.0;
        surface.set_fill(Some(fill(75, 75, 75)));
        surface.draw_text(
            Point::from_xy(405.0, y),
            regular.clone(),
            8.0,
            label,
            false,
            TextDirection::Auto,
        );

        surface.set_fill(Some(fill(0, 0, 0)));
        if matches!(label, "Total") {
            surface.draw_text(
                Point::from_xy(455.0, y),
                bold.clone(),
                11.0,
                &value,
                false,
                TextDirection::Auto,
            );
        } else {
            surface.draw_text(
                Point::from_xy(455.0, y),
                regular.clone(),
                11.0,
                &value,
                false,
                TextDirection::Auto,
            );
        }
    }

    surface.finish();
    page.finish();

    let pdf = document.finish().expect("finish PDF");

    let mut default_name = invoice.title.to_ascii_lowercase();
    default_name.push_str(".pdf");
    let file_name = invoice.output.unwrap_or(default_name);
    fs::write(file_name, pdf).expect("write PDF");
}
