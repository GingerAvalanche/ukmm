use std::{env, fs, path::Path};

use anyhow::{Context, Result, anyhow};
use roead::{
    aamp::{Name, Parameter, ParameterIO, ParameterListing},
    sarc::Sarc,
    yaz0,
};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .context("usage: cargo run -p uk-content --example dump_recipe <path> [entry]")?;
    if input == "--hash" {
        for name in args {
            let name = Name::from_str(&name);
            println!("{name}: hash={}", name.hash());
        }
        return Ok(());
    }
    let entry = args.next();
    let data = load_recipe_bytes(&input, entry.as_deref())?;
    dump_recipe(&data)?;
    Ok(())
}

fn load_recipe_bytes(path: &str, entry: Option<&str>) -> Result<Vec<u8>> {
    let path = Path::new(path);
    if path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("brecipe"))
        .unwrap_or(false)
    {
        return fs::read(path).with_context(|| anyhow!("failed to read {}", path.display()));
    }

    let raw = fs::read(path).with_context(|| anyhow!("failed to read {}", path.display()))?;
    if path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("sbactorpack") || ext.eq_ignore_ascii_case("sarc"))
        .unwrap_or(false)
    {
        let entry = entry.context(
            "SARC input requires an entry path, e.g. Actor/Recipe/Armor_421_Head.brecipe",
        )?;
        let decompressed = yaz0::decompress(raw).context("failed to decompress Yaz0 data")?;
        let sarc = Sarc::new(decompressed).context("failed to parse SARC container")?;
        let data = sarc.get_data(entry).context("missing entry inside SARC")?;
        return Ok(data.to_vec());
    }

    Err(anyhow!(
        "unsupported input type {}; provide a .brecipe file or SARC container",
        path.display()
    ))
}

fn dump_recipe(data: &[u8]) -> Result<()> {
    let pio = ParameterIO::from_binary(data).context("failed to parse ParameterIO")?;
    let header = pio
        .object("Header")
        .context("recipe missing Header object")?;
    println!("Header:");
    for (key, value) in header.iter() {
        print_parameter(&key.to_string(), value);
    }
    println!();
    for (table_name, table) in &pio.objects().0 {
        let table_name = table_name.to_string();
        if table_name == "Header" {
            continue;
        }
        println!("Table {}:", table_name);
        for (key, value) in table.iter() {
            print_parameter(&key.to_string(), value);
        }
        println!();
    }
    Ok(())
}

fn print_parameter(name: &str, value: &Parameter) {
    match value {
        Parameter::String64(v) => println!("  {name}: String64({})", v.as_str()),
        Parameter::I32(v) => println!("  {name}: I32({v})"),
        Parameter::U32(v) => println!("  {name}: U32({v})"),
        Parameter::F32(v) => println!("  {name}: F32({v})"),
        Parameter::Bool(v) => println!("  {name}: Bool({v})"),
        other => println!("  {name}: {:?}", other),
    }
}
