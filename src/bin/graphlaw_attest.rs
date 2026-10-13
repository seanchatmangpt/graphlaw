//! `graphlaw-attest`: sign/verify an arbitrary file with `graphlaw::attest`.
//!
//! ```text
//! graphlaw-attest sign   --key-seed-hex <hex> --in <file> [--out <attestation.json>]
//! graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]
//! graphlaw-attest pubkey --key-seed-hex <hex>
//! ```
//!
//! `sign` writes the canonical attestation JSON (and prints it). `verify`
//! re-reads the file and attestation, runs `attest::verify_bytes`, and prints
//! `VERIFIED` or `REFUSED: <error>`. Exit codes: 0 ok, 1 refused, 2 usage/IO.

use std::path::PathBuf;
use std::process::ExitCode;

use graphlaw::attest::{sign_bytes, verify_bytes, Attestation, SigningKey, TrustedKeys};

struct Args {
    mode: Mode,
    seed_hex: Option<String>,
    input: Option<PathBuf>,
    out: Option<PathBuf>,
    attestation_path: Option<PathBuf>,
    keys: Vec<String>,
}

enum Mode {
    Sign,
    Verify,
    Pubkey,
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("graphlaw-attest: {msg}");
    eprintln!(
        "usage: graphlaw-attest sign --key-seed-hex <hex> --in <file> [--out <att.json>]\n       \
         graphlaw-attest verify --in <file> --attestation <json> --trusted-key <hex> [...]\n       \
         graphlaw-attest pubkey --key-seed-hex <hex>"
    );
    ExitCode::from(2)
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let mode = match it.next().as_deref() {
        Some("sign") => Mode::Sign,
        Some("verify") => Mode::Verify,
        Some("pubkey") => Mode::Pubkey,
        _ => return Err("first argument must be `sign`, `verify` or `pubkey`".into()),
    };
    let (mut seed_hex, mut input, mut out, mut attestation_path) = (None, None, None, None);
    let mut keys = Vec::new();
    while let Some(flag) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("`{flag}` needs a value"));
        match flag.as_str() {
            "--key-seed-hex" => seed_hex = Some(val()?),
            "--in" => input = Some(PathBuf::from(val()?)),
            "--out" => out = Some(PathBuf::from(val()?)),
            "--attestation" => attestation_path = Some(PathBuf::from(val()?)),
            "--trusted-key" => keys.push(val()?),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    match mode {
        Mode::Pubkey if seed_hex.is_none() => Err("pubkey needs --key-seed-hex".into()),
        Mode::Pubkey => Ok(Args { mode, seed_hex, input: None, out, attestation_path, keys }),
        _ => {
            let input = input.ok_or("missing --in")?;
            match mode {
                Mode::Sign if seed_hex.is_none() => Err("sign needs --key-seed-hex".into()),
                Mode::Sign => Ok(Args { mode, seed_hex, input: Some(input), out, attestation_path, keys }),
                Mode::Verify if keys.is_empty() => Err("verify needs at least one --trusted-key".into()),
                Mode::Verify => Ok(Args { mode, seed_hex, input: Some(input), out, attestation_path, keys }),
                Mode::Pubkey => unreachable!(),
            }
        }
    }
}

fn read(p: &std::path::Path) -> Result<Vec<u8>, String> {
    std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))
}

fn run(a: Args) -> Result<ExitCode, String> {
    if let Mode::Pubkey = a.mode {
        let seed = graphlaw::attest::hex_decode(a.seed_hex.as_deref().unwrap())
            .ok_or("--key-seed-hex: not hex")?;
        let seed: [u8; 32] = seed.try_into().map_err(|_| "--key-seed-hex: need 32 bytes")?;
        println!("{}", SigningKey::from_seed(seed).verifying_key().to_hex());
        return Ok(ExitCode::SUCCESS);
    }
    let payload = read(a.input.as_deref().unwrap())?;
    match a.mode {
        Mode::Pubkey => unreachable!("handled above"),
        Mode::Sign => {
            let seed = graphlaw::attest::hex_decode(a.seed_hex.as_deref().unwrap())
                .ok_or("--key-seed-hex: not hex")?;
            let seed: [u8; 32] = seed.try_into().map_err(|_| "--key-seed-hex: need 32 bytes")?;
            let key = SigningKey::from_seed(seed);
            let att = sign_bytes(&key, &payload);
            let json = att.to_json();
            let text = format!("{json}\n");
            match &a.out {
                Some(p) => {
                    std::fs::write(p, text.as_bytes()).map_err(|e| format!("{}: {e}", p.display()))?;
                    println!("{json}");
                }
                None => println!("{json}"),
            }
        }
        Mode::Verify => {
            let ap = a
                .attestation_path
                .ok_or("verify needs --attestation")?;
            let text = read(&ap)?;
            let text = String::from_utf8(text).map_err(|_| "attestation file is not UTF-8")?;
            let att = Attestation::from_json(text.trim())
            .map_err(|e| format!("attestation: {e}"))?;
            let trusted = TrustedKeys::from_hex(&a.keys).map_err(|e| format!("--trusted-key: {e}"))?;
            if let Err(e) = verify_bytes(&payload, &att, &trusted) {
                println!("REFUSED: {e}");
                return Ok(ExitCode::from(1));
            }
            println!("VERIFIED");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match parse_args() {
        Err(m) => usage(&m),
        Ok(a) => run(a).unwrap_or_else(|m| usage(&m)),
    }
}
