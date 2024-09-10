use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr32BE;

pub fn encrypt<const N: usize>(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8; N]) -> [u8; N] {
    let mut cipher = Ctr32BE::<Aes128>::new(key.into(), iv.into());
    let mut ciphertext = plaintext.to_vec();
    cipher.apply_keystream(&mut ciphertext);

    if cfg!(debug_assertions) {
        let mut decrypted_text = ciphertext.clone();
        let mut decrypt_cipher = Ctr32BE::<Aes128>::new(key.into(), iv.into());
        decrypt_cipher.apply_keystream(&mut decrypted_text);
        assert_eq!(plaintext.as_slice(), decrypted_text);
    }

    let mut answer = [0u8; N];
    answer.copy_from_slice(&ciphertext[..]);
    answer
}
