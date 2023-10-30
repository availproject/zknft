use anyhow::{Error, anyhow};

pub fn hex_string_to_u8_array(hex_string: &str) -> Result<[u8; 32], Error> {
  let hex_string = if hex_string.starts_with("0x") {
    &hex_string[2..] // Remove the "0x" prefix
  } else {
      hex_string
  };

    let bytes = match hex::decode(hex_string) {
      Ok(i) => i, 
      Err(e) => return Err(anyhow!("{:?}", e)),
    };
    
    if bytes.len() != 32 {
        return Err(anyhow!("Hexadecimal string must represent exactly 32 bytes"));
    }
  
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
  
    Ok(array)
}

pub fn u8_array_to_hex_string(data: &[u8]) -> String {
  let hex_string: String = data.iter().map(|byte| format!("{:02X}", byte)).collect();
  hex_string
}

pub fn hex_string_to_u64(hex_string: &str) -> Result<u64, Error> {
  match u64::from_str_radix(hex_string, 16) {
    Ok(i) => Ok(i), 
    Err(e) => Err(anyhow!("{:?}", e)),
  }
}