use std::collections::HashMap;

use aes::Aes128;
use ark_groth16::prepare_verifying_key;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr32BE;
use vc_prove::{
    circuit::circom_builder,
    groth16::{prove, setup, verify},
    warmup_current_thread, Signal,
};
use rand::Rng;

struct ByteArray<const N: usize>([u8; N]);

impl<const N: usize> Signal for ByteArray<N> {
    fn to_signal(&self) -> Vec<num_bigint::BigInt> {
        self.0
            .iter()
            .map(|x| num_bigint::BigInt::from(*x))
            .collect()
    }
}

fn random_array<const N: usize>() -> [u8; N]{
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

    // 定义 128 位密钥和 128 位 IV (Nonce)
    let key = b"verysecretkey124"; // 16 字节密钥 (128 位)
    let iv = b"uniqueiv1234ffff"; // 16 字节 IV (nonce)
    let plaintext = random_array::<128>();
    let ciphertext = encrypt(key, iv, &plaintext);

    let mut input = HashMap::new();
    input.insert("plainText", ByteArray(plaintext).to_signal());
    input.insert("iv", ByteArray(*iv).to_signal());
    input.insert("key", ByteArray(*key).to_signal());

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

    let success = verify(&vk, &proof, &ByteArray(ciphertext)).unwrap();
    assert!(success);

    println!("Done");
}
