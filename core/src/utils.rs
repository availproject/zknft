use anyhow::{Error, anyhow};

pub fn hex_string_to_u8_array(hex_string: &str) -> Result<[u8; 32], Error> {
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

pub fn hex_string_to_u64(hex_string: &str) -> Result<u64, Error> {
  match u64::from_str_radix(hex_string, 16) {
    Ok(i) => Ok(i), 
    Err(e) => Err(anyhow!("{:?}", e)),
  }
}