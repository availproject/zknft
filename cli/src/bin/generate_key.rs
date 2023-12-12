use std::fs::File;
use std::io::Write;
use std::io::Error;

fn main() -> Result<(), Error> {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);

    let keypair_bytes = signing_key.to_bytes();

    // Create a JSON object containing keypair_bytes
    let json_data = serde_json::json!({
      "keypair_bytes": keypair_bytes,
    });

    // Create a new JSON file and write the data to it
    let mut file = File::create("keypair.json")?;
    file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes())?;

    Ok(())
}