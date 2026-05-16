# invoice-rs

A small command-line tool for generating clean, printable PDF invoices. Written in Rust on top of [krilla](https://crates.io/crates/krilla) for the PDF side and [clap](https://crates.io/crates/clap) for the CLI.

I built this because every "invoice generator" I tried online either wanted my email, added a watermark, or shipped a 40MB Electron app. This one is a single binary, takes a few flags, and drops a PDF in your working directory.

## Requirements

- Rust (stable, edition 2024)
- The Inter font files, expected at `./Inter/Inter Hinted for Windows/Desktop/Inter-Regular.ttf` and `Inter-Bold.ttf` relative to where you run the binary. Download from [rsms.me/inter](https://rsms.me/inter/) and unzip into the project root.

## Build

```bash
cargo build --release
```

The release profile is tuned for a small binary (`opt-level = "z"`, LTO, stripped). The output lands at `target/release/invoice-rs`.

For day-to-day use you can just run `cargo run --` and let cargo handle the build.

## Usage

The only subcommand right now is `generate`:

```bash
cargo run -- generate \
  --from "Acme Studios" \
  --to "Globex Corporation" \
  --id "2026-001" \
  --items "Design work" --quantities 10 --prices 75.0 \
  --items "Revisions"   --quantities 2  --prices 60.0 \
  --tax 0 --discount 0 \
  --currency USD \
  --output acme-001.pdf
```

That writes `acme-001.pdf` next to your terminal. If you skip `--output`, the file is named after the title (default: `invoice.pdf`).

### Flags

| Flag | Default | Notes |
| --- | --- | --- |
| `--id` | today's date, `YYYYMMDD` | Invoice number printed next to the title. |
| `--title` | `INVOICE` | Big bold heading. |
| `--from` | `Project Folded, Inc.` | Sender line at the top. |
| `--to` | `Untitled Corporation, Inc.` | "Bill to" name. |
| `--date` | today | Free-form date string. |
| `--due` | today + 14 days | Free-form date string. |
| `--items` | `Paper Cranes` | Pass once per line item. |
| `--quantities` | `2` | One per item, same order. |
| `--prices` | `25.0` | One per item, same order. |
| `--tax` | `0.0` | Added on top of subtotal. |
| `--discount` | `0.0` | Subtracted from subtotal. |
| `--currency` | `INR` | One of: USD, EUR, GBP, JPY, CNY, INR, RUB, KRW, BRL, SGD. |
| `--note` | empty | Free-form note (currently stored, not yet rendered). |
| `--logo` | empty | Path to a logo (stored, not yet rendered). |
| `--output` | `<title>.pdf` | Output filename. |

`--items`, `--quantities`, and `--prices` are positionally matched, so the order you pass them in is the order they show up in the table.

## What it looks like

The layout is intentionally minimal: sender at the top, a thin divider, the title and invoice number, a "BILL TO" block, then a borderless items table with QTY / RATE / AMOUNT columns, and a Subtotal / Total / Due Date summary at the bottom.

## Known limits

- The page is fixed-size A4 and there is no pagination yet. If you push more than ~25 line items, rows past the bottom margin will get clipped. Pagination is on the list.
- `--logo` and `--note` are parsed but not yet drawn on the PDF.
- The Inter font path is hardcoded; it has to live in the working directory.

## Project layout

```
src/
  main.rs      CLI parsing and PDF layout
  invoice.rs   Invoice struct, defaults, Currency enum
```

## License

Personal project, no license attached yet. Ask before shipping it somewhere.
