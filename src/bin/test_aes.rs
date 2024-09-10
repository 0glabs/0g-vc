use std::collections::HashMap;

use aes::Aes128;
use ark_groth16::prepare_verifying_key;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr32BE;
use rand::Rng;
use vc_prove::types::ByteArray;
use vc_prove::{
    circuit::circom_builder,
    groth16::{prove, setup, verify},
    warmup_current_thread, Signal,
};

fn random_array<const N: usize>() -> [u8; N] {
    let mut rng = rand::thread_rng();
    let mut array = [0u8; N];
    rng.fill(&mut array[..]);
    array
}

fn encrypt<const N: usize>(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8; N]) -> [u8; N] {
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

fn main() {
    warmup_current_thread();
    let task_name = "test_aes";

    let key = b"verysecretkey123"; 
    let iv = &hex::decode("756e697175656976313233346666ffff").unwrap().try_into().unwrap();
    let plaintext = random_array::<111>();
    let ciphertext = encrypt(key, iv, &plaintext);

    let mut input = HashMap::new();
    input.insert("plainText", ByteArray::new(plaintext).to_signal());
    input.insert("iv", ByteArray::new(*iv).to_signal());
    input.insert("key", ByteArray::new(*key).to_signal());

    let circom = circom_builder(&"output".into(), &task_name);

    // 1. Setup Params
    println!("Generate params");
    let pk = setup(&circom).unwrap();
    let vk = prepare_verifying_key(&pk.vk);

    // 2. Prove
    println!("Prove");
    let proof = prove(&pk, &circom, input).unwrap();

    // 3. Verify
    println!("Verify");

    let success = verify(&vk, &proof, &ByteArray::new(ciphertext)).unwrap();
    assert!(success);

    println!("Done");
}
