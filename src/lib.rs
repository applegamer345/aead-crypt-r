use arrayref::array_ref;
use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, Aead, NewAead, generic_array::GenericArray},
    XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use std::{
    fs::{self, File},
    io::{Read, Write, Seek}, hash::{self, Hash},
};

use sha256;



//-> Result<(), anyhow::Error>
pub fn encrypt_data_file(
    source_file_path: &str,
    key: &[u8; 32],
    nonce: &[u8; 19],
) -> Result<(), anyhow::Error> {
    let dist_file_path = source_file_path.to_owned() + ".enc";
    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    const BUFFER_LEN: usize = 500;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut source_file = File::open(source_file_path)?;
    let mut dist_file = File::create(dist_file_path)?;
    dist_file.write(nonce).expect("Couldn't write Nonce file is useless!");

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
            break;
        }
    }
    Ok(())
}


/// Get the first 3 elements of `bytes` as a reference to an array
/// **Panics** if `bytes` is too short.


pub fn password_to_key(password: String) -> [u8;32] {
    // returns a 32 Byte key.

    let v = sha256::digest(&*password);

    return array_ref![v.split_at(16).1.split_at(32).0.as_bytes(), 0, 32].to_owned();
}

pub fn decrypt_data_file(
    encrypted_file_path: &str,
    key: &[u8; 32],
    nonce: &[u8; 19],
) -> Result<(), anyhow::Error> {
    let dist = encrypted_file_path.replace(".enc", ""); 
    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_decryptor = stream::DecryptorBE32::from_aead(aead, nonce.as_ref().into());

    const BUFFER_LEN: usize = 500 + 16;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut encrypted_file = File::open(encrypted_file_path)?;
    let mut dist_file = File::create(dist)?;
    encrypted_file.seek(std::io::SeekFrom::Start(19)).expect("Couldn't skip Nonce!");

    loop {
        let read_count = encrypted_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let plaintext = stream_decryptor
                .decrypt_next(buffer.as_slice())
                .map_err(|err| anyhow!("Decrypting large file: {}", err))?;
            dist_file.write(&plaintext)?;
        } else if read_count == 0 {
            break;
        } else {
            let plaintext = stream_decryptor
                .decrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Decrypting large file: {}", err))?;
            dist_file.write(&plaintext)?;
            break;
        }
    }

    Ok(())
}

pub fn rand_key_nonce() -> ([u8;32], [u8;19]) {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);
    return  (key,nonce);
}
