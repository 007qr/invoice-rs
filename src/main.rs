mod invoice;

use std::fs;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use invoice::Invoice;
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Size, Transform};
use krilla::image::Image;
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

fn text_width(font_bytes: &[u8], text: &str, size_pt: f32) -> f32 {
    let face = ttf_parser::Face::parse(font_bytes, 0).expect("parse font");
    let upem = face.units_per_em() as f32;
    let scale = size_pt / upem;
    let mut width = 0.0_f32;
    for ch in text.chars() {
        if let Some(gid) = face.glyph_index(ch) {
            if let Some(adv) = face.glyph_hor_advance(gid) {
                width += adv as f32 * scale;
            }
        }
    }
    width
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
    let regular_bytes =
        fs::read("./Inter/Inter Hinted for Windows/Desktop/Inter-Regular.ttf").unwrap();
    let medium_bytes =
        fs::read("./Inter/Inter Hinted for Windows/Desktop/Inter-Medium.ttf").unwrap();
    let bold_bytes = fs::read("./Inter/Inter Hinted for Windows/Desktop/Inter-Bold.ttf").unwrap();

    let regular = Font::new(Arc::new(regular_bytes.clone()).into(), 0).expect("load Inter-Regular");
    let medium = Font::new(Arc::new(medium_bytes).into(), 0).expect("load Inter-Medium");
    let bold = Font::new(Arc::new(bold_bytes).into(), 0).expect("load Inter-Bold");

    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::from_wh(page_w, page_h).unwrap());
    let mut surface = page.surface();

    // krilla uses top-down y coords, with text positioned at the baseline.
    // Track a cursor that walks down the page.
    let mut y = margin;

    // 1. "From" line — 12pt, dark grey.
    y += 12.0; // baseline drop for 12pt
    // If logo available then show it at top
    if !invoice.logo.is_empty() {
        let bytes = fs::read(&invoice.logo).expect("read logo");
        let data = Arc::new(bytes).into();
        let lower = invoice.logo.to_ascii_lowercase();
        let image = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            Image::from_jpeg(data, false).expect("decode jpeg logo")
        } else {
            Image::from_png(data, false).expect("decode png logo")
        };

        let (iw, ih) = image.size();
        let max_h = 35.0_f32;
        let scale = (max_h / ih as f32).min(1.0);
        let draw_w = iw as f32 * scale;
        let draw_h = ih as f32 * scale;

        surface.push_transform(&Transform::from_translate(margin, y - 12.0));
        surface.draw_image(image, Size::from_wh(draw_w, draw_h).unwrap());
        surface.pop();

        // Push the rest of the layout below the logo.
        y += draw_h + 8.0;
    }

    y += 8.0;
    surface.set_fill(Some(fill(100, 100, 100)));
    surface.draw_text(
        Point::from_xy(margin, y),
        regular.clone(),
        13.0,
        &invoice.from,
        false,
        TextDirection::Auto,
    );

    // 2. Horizontal rule — matches the width of the "From" text above it.
    y += 30.0;
    let from_w = text_width(&regular_bytes, &invoice.from, 13.0);
    let rule = {
        let mut pb = PathBuilder::new();
        pb.move_to(margin, y);
        pb.line_to(margin + from_w, y);
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
    surface.set_fill(Some(fill(150, 150, 150)));
    for (x, label) in [
        (col_item, "ITEM"),
        (col_qty, "QTY"),
        (col_rate, "RATE"),
        (col_amount, "AMOUNT"),
    ] {
        surface.draw_text(
            Point::from_xy(x, y),
            medium.clone(),
            10.0,
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
        rows.insert(
            rows.len() - 2,
            ("Discount", format!("{:.2}", invoice.discount)),
        )
    }

    y += 24.0 + 240.0;
    // Notes
    surface.set_fill(Some(fill(150, 150, 150)));
    surface.draw_text(
        Point::from_xy(margin, y),
        medium.clone(),
        10.0,
        "Notes",
        false,
        TextDirection::Auto,
    );

    surface.set_fill(Some(fill(0, 0, 0)));
    surface.draw_text(
        Point::from_xy(margin, y + 20.0),
        medium.clone(),
        10.0,
        &invoice.note,
        false,
        TextDirection::Auto,
    );

    y -= 24.0;
    for (label, value) in rows {
        y += 24.0;
        surface.set_fill(Some(fill(150, 150, 150)));
        surface.draw_text(
            Point::from_xy(405.0, y),
            medium.clone(),
            10.0,
            label,
            false,
            TextDirection::Auto,
        );

        surface.set_fill(Some(fill(0, 0, 0)));
        if matches!(label, "Total") {
            surface.draw_text(
                Point::from_xy(475.0, y),
                bold.clone(),
                10.0,
                &value,
                false,
                TextDirection::Auto,
            );
        } else {
            surface.draw_text(
                Point::from_xy(475.0, y),
                regular.clone(),
                10.0,
                &value,
                false,
                TextDirection::Auto,
            );
        }
    }

    let footer_y = page_h - margin;
    let footer_label = format!("Invoice #{}", invoice.id);
    let label_w = text_width(&regular_bytes, &footer_label, 10.0);
    let gap = 8.0_f32;

    surface.set_stroke(None);
    surface.set_fill(Some(fill(100, 100, 100)));
    surface.draw_text(
        Point::from_xy(margin, footer_y),
        regular.clone(),
        10.0,
        &footer_label,
        false,
        TextDirection::Auto,
    );

    let rule_y = footer_y - 3.5;
    let rule_start = margin + label_w + gap;
    let footer_rule = {
        let mut pb = PathBuilder::new();
        pb.move_to(rule_start, rule_y);
        pb.line_to(page_w - margin, rule_y);
        pb.finish().unwrap()
    };
    surface.set_fill(None);
    surface.set_stroke(Some(Stroke {
        paint: rgb::Color::new(225, 225, 225).into(),
        width: 1.0,
        ..Default::default()
    }));
    surface.draw_path(&footer_rule);

    surface.finish();
    page.finish();

    let pdf = document.finish().expect("finish PDF");

    let mut default_name = invoice.title.to_ascii_lowercase();
    default_name.push_str(".pdf");
    let file_name = invoice.output.unwrap_or(default_name);
    fs::write(file_name, pdf).expect("write PDF");
}
