# Background

Provide ZK tools that allow 0G users to use ZK tools to display and prove arbitrary granularity of fields in verifiable certificates.
For example, a user has a verifiable certificate on 0G that includes fields such as name, age, date of birth, and education level code. The user wants to show others that he/she was born after 2000.

# Design

## Certificate Format

Verifiable certificates should contain at least a serial_no field to determine the ownership of the certificate. Users who know the serial_no own the certificate. Other fields in the certificate vary depending on what the certificate is and can be customized. The serial_no also serves to prevent exhaustive guessing of private fields. Here, an academic certificate is used as an example to illustrate the certificate format.

An academic certificate contains the fields of name, age, date of birth, and education level code:
```rust
pub struct VC {
    name: String,
    age: u8,
    birth_date: String,
    edu_level: u8,
    serial_no: String,
}
```
The fields of the certificate should be properly encoded to facilitate subsequent hashing. The format of the encoded VC is as follows:
```rust
pub struct EncodedVC {
    name: Vec<u8>, // 4+16bytes
    age: Vec<u8>, // 3+1bytes
    birth_date: Vec<u8>, // 5+8bytes
    edu_level: Vec<u8>, // 3+1bytes
    serial_no: Vec<u8>, // 5+32bytes
}
```

- name: The prefix is "name", and the main part is encoded using UTF-8 with a length of 16 bytes.
- age: The prefix is "age", and the main part uses the u8 type.
- birth_date: The prefix is "birth", and the main part is represented using a Unix timestamp with a length of 8 bytes (u64).
- edu_level: The prefix is "edu", and the main part uses the u8 type.
- serial_no: The prefix is "serial", and the main part uses the hexString type with a length of 32 bytes.

The effective data part of the encoded VC is 79 bytes, padded to 256 bytes.
![Screenshot 2024-05-21 at 14.53.23.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716274410323-b8a8e4fd-f9c2-4e48-9336-e34f43ee3468.png#averageHue=%23ededed&clientId=u4e87bca3-f673-4&from=drop&id=ub57c7a25&originHeight=286&originWidth=2344&originalType=binary&ratio=2&rotation=0&showTitle=false&size=133204&status=done&style=none&taskId=ude550725-cda6-40f7-b45d-11e6a6e2a3d&title=)

The leaf of the VC tree is obtained by hashing the EVC (Encoded VC), and the root hash is calculated upward from multiple VCs. The hash function used here is Keccak. The calculation from EVC to leaf uses Keccak(256*8, 256), and the calculation of intermediate nodes uses Keccak(256*2, 256).

![Screenshot 2024-05-21 at 15.14.43.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716275690140-ed2a21fa-24e5-4950-af8e-569291af3fd3.png#averageHue=%23f4f4f4&clientId=u4e87bca3-f673-4&from=drop&id=ue90c49cb&originHeight=1024&originWidth=1652&originalType=binary&ratio=2&rotation=0&showTitle=false&size=271073&status=done&style=none&taskId=u5071e7dd-9ca0-4985-a2de-81f78e32398&title=)

## Circuit Constraints

The root hash of the VC tree is stored on 0G. When a user wants to prove field properties (such as being born after 2000), they can prove:

1. The birth_date field is greater than (encoded) 20000101.
2. The VC tree root hash is equal to the hash stored on 0G.

If both of the above assertions hold, the proof is valid.

# Practice

## Quick Start

1. Install Yarn

2. Install circom and build the circuit
```bash
yarn
yarn build
```

3. Run the demo
```bash
cargo run -r --bin example
```
If you want to enable the GPU, you can enable the cuda feature
```bash
cargo run -r --bin example --features="cuda"
```

## Project Dependencies

- Circom compiler
- Circomlib
- Groth16-gpu

## Project Structure

The project directory is as follows:

![Screenshot 2024-05-21 at 15.27.24.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716276450567-4af591fd-bdd0-453a-9055-a2da39ae3611.png#averageHue=%23222222&clientId=u4e87bca3-f673-4&from=drop&height=264&id=w2B6T&originHeight=668&originWidth=382&originalType=binary&ratio=2&rotation=0&showTitle=false&size=71638&status=done&style=none&taskId=ue3ac2436-99f1-4814-967f-3d367829a99&title=&width=151)

Where:
- circuits are circom circuits, mainly including VC circuits, Keccak hash circuits, Merkel proof circuits, and some common utility circuits. Note that some modifications have been made to the Keccak hash circuit here to accept inputs of arbitrary length. The original inputs length cannot exceed blocksize=136bytes, which does not meet the requirement of our LeafHasher (input length is 256 bytes). This may cause the hash calculated in the circuit to be inconsistent with the hash calculated externally.
- fronted script is the compilation logic for the circom circuit, using the circom compiler.
- circomlib circuit library is some library circuits provided by the official.
- output, the output of the frontend part is saved in this directory, mainly the generated r1cs files and witness calculator files.
- src is the main code logic, which is divided into three parts: certificate formatting, proof frontend, and proof backend:

- - Certificate formatting is responsible for encoding the user-readable certificate format into the proof input format.

```rust
#[derive(Debug, Clone)]
pub struct VC {
    name: String,
    age: u8,
    birth_date: String,
    edu_level: u8,
    serial_no: String,
}

#[derive(Debug, Clone)]
pub struct EncodedVC {
    name: Vec<u8>,
    age: Vec<u8>,
    birth_date: Vec<u8>,
    edu_level: Vec<u8>,
    serial_no: Vec<u8>,
}
```

- - The proof frontend uses the circom compiler and actually calls the frontend script to compile the input .circom file into the proof backend input r1cs and generate witness calculation code.

```rust
pub fn compile_circuit(input_file: &str, output_dir: &str) {
    // 指定要执行的 Shell 文件路径
    let project_dir = env::current_dir().expect("Failed to get current directory");
    let script_path = project_dir.join("frontend.sh");

    // 使用 Command::new 创建一个新的命令
    let mut cmd = Command::new("sh");

    // 将要执行的 Shell 文件路径和参数传递给 sh 命令
    cmd.arg(script_path).arg(input_file).arg(output_dir);

    // 执行命令
    let output = cmd.output().expect("Failed to execute shell script");

    // 打印命令的输出
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
```

- - Proof backend: Calculate witness

```rust
pub fn cal_witness(
    wtns: impl AsRef<Path>,
    r1cs: impl AsRef<Path>,
    inputs: HashMap<String, Vec<BigInt>>,
) -> Result<(CircomCircuit<Bn254>, Vec<Fr>), Box<dyn std::error::Error>>
{
    let cfg = CircomConfig::<Bn254>::new(wtns, r1cs).unwrap();
    let mut builder = CircomBuilder::new(cfg);

    // 遍历输入参数的 HashMap，并为每个输入调用 push_input 函数
    for (name, values) in inputs {
        for value in values {
            builder.push_input(&name, value);
        }
    }
    // println!("builder inputs:{:?}", builder.inputs);
    let circom = builder.build()?;
    let pub_in = circom.get_public_inputs().unwrap();
    println!("public inputs:{:?}", pub_in);
    Ok((circom, pub_in))
}
```

- - Generate and verify proofs using the corresponding backend system

```rust
pub fn gen_proof(
    circuit: CircomCircuit<Bn254>,
    pk: &ProvingKey<Bn254>,
    rng: &mut impl Rng,
) -> Proof<Bn254> {
    Groth16::create_random_proof_with_reduction(circuit, pk, rng).unwrap()
}

pub fn ver_proof(pk: &ProvingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &Vec<Fr>) -> bool {
    let pvk = prepare_verifying_key(&pk.vk);
    Groth16::verify_proof(&pvk, proof, public_inputs).unwrap()
}
```

## Note

The project provides an example file src/main.rs, which gives a proof that the birth_data field of a certificate is less than March 4, 2000. It should be noted that it is assumed here that the VC tree stored on 0G is as shown in the 3-layer VC tree in the Certificate Format section above, with only one VC stored and the rest being empty.
