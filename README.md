# Verifiable Certificates Tool for Decentralized Storage

Provide ZK tools that allow 0G users to use ZK tools to display and prove arbitrary granularity of fields in verifiable certificates.

## Installation

### Prerequisites

Before installing the project, ensure your system meets the following requirements on Ubuntu.

- **Install Rust:**
  Use the official installation script:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  Install the specific version used
  ```bash
  rustup install nightly-2024-02-04
  ```
  Configure your current shell to start using Rust:
  ```bash
  source $HOME/.cargo/env
  ```

- **Install Node.js (>= v18):**
  Install using NodeSource binary distributions:
  ```bash
  curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
  sudo apt-get install -y nodejs
  ```

- **Install Yarn:**
  Install via npm after Node.js:
  ```bash
  npm install --global yarn
  ```

- **Install CUDA (optional, for GPU acceleration):**
  Download and install the CUDA 12.4 toolkit from the [NVIDIA CUDA Toolkit webpage](https://developer.nvidia.com/cuda-12-4-1-download-archive).

- **Install libsnark-rust dependencies (optional, for performance comparison):**
  Install dependencies:
  ```bash
  sudo apt update
  # libsnark dependencies
  sudo apt install build-essential cmake git libgmp3-dev python3-markdown libboost-program-options-dev libssl-dev python3 pkg-config
  # gmp-mpfr-sys dependencies
  sudo apt install diffutils gcc m4 make
  # libsnark must work with the specific gcc version
  sudo apt install gcc-10 g++-10
  ```

### Installation

Build the project using Cargo.

```bash
cargo build --release
```

### Compilation Options

This project offers the following compilation:

- **CUDA Support:**
  To enable GPU acceleration using CUDA, make sure the CUDA dependencies are correctly installed as described in the Prerequisites section. Compile the project with the `--features cuda` flag:
  ```bash
  cargo build --release --features cuda
  ```

- **Trace Logging:**
  For more verbose output and trace logging for groth16 process, enable the trace feature by using the `--features trace` flag:
  ```bash
  cargo build --release --features trace
  ```

- **Libsnark FFI:**
  For comparing performance with libsnark FFI, enable the libsnark feature by using the `--features libsnark` flag:
  ```bash
  cargo build --release --features libsnark
  ```
  **Note:** If you get stucked on updating submodules, try the following command:
  ```bash
  git config --global url."https://".insteadOf git://
  ```

Each feature can be enabled individually or combined depending on the development and debugging needs. For combined features, use:
```bash
cargo build --release --features "cuda,trace"
```

### Running

Before running the application, it is necessary to prepare the environment and generate required parameters. Follow these steps to set up:

- **Setup:**
   Install all JavaScript dependencies, build the necessary components, and generate parameters:
   ```bash
   yarn
   yarn build
   yarn setup
   ```
   **Note:** This process may take 15 minutes or even longer, depending on your CPU performance. This process may consume more than 64 GB of memory. Please ensure that you have sufficient memory/virtual memory, or use a high-performance machine to generate the parameters and then copy them to others.

- **Running Example Code:**
  To run the [example code](./src/bin/example.rs), especially if you wish to enable CUDA features, compile the project using the following command:
  ```bash
  cargo build --release --bin example --features trace
  ```
  Add `cuda` feature if you have configured CUDA support and wish to use GPU acceleration.

  **Note:** If you enable the CUDA feature, we highly recommend using a fixed thread to call the proof function and communicate with other threads via channels, rather than calling the proof function in every thread.


- **Running libsnark Comparison Code:**
  If you have installed all necessary dependencies for libsnark as per the Prerequisites section, compile and run the libsnark comparison code:
  ```bash
  cargo build --release --bin libsnark --features libsnark
  ```

## Code Details and Developer Interfaces

This proof of concept (PoC) demonstrates a simplified example of verifiable certificates using the circom circuit framework, input and output construction, Groth16 proof generation, CUDA acceleration, and a performance comparison with libsnark.

### Verifiable Certificates

Verifiable certificates should contain at least a serial_no field to determine the ownership of the certificate. Users who know the serial_no own the certificate. Other fields in the certificate vary depending on what the certificate is and can be customized. The serial_no also serves to prevent exhaustive guessing of private fields. Here, an academic certificate is used as an example to illustrate the certificate format. 

An academic certificate contains the fields of name, age, date of birth, and education level code:
```rust
pub struct VC {
    // <= 16 bytes, UTF-8 supported
    name: String, 
    age: u8,
    birth_date: NaiveDate,
    edu_level: u8,
    // <= 32 bytes
    serial_no: Vec<u8>,
}
```

Here is an example code constructing `VC` from a json string. Note
```rust
let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "19991231", "edu_level": 4, "serial_no": "3921b15ceb8f4be8891d1de1e64af044"}"#;
let vc = VC::from_json(vc_json).unwrap();
```

Rust developers can also construct `VC` directly with `VC::new(...)`. Please refer to `NativeDate`'s [document](https://docs.rs/chrono/latest/chrono/) to initialize it.

### Publish Verification Certificates on Chain

The VC is encoded into a content of 79 bytes in a unique format, followed by the computation of its keccak256 hash value (32 bytes). The hash is then uploaded to the decentralized storage flow (not covered in this repo). Since the minimum unit of decentralized storage is 256 bytes, it is padded with zeros to meet this minimum length.

### Type Interfaces

Despite zero-knowledge proofs requiring large integers from finite fields as inputs, developers do not need to concern themselves with the details of these conversions. Simply provide inputs using the predefined types `ProveInput` and `VerifyInput`([source code](./src/types/input.rs)). 

``` rust
pub struct ProveInput {
    data: VC,
    birthdate_threshold: NaiveDate,
    merkle_proof: Vec<H256>,
    // The position index of the corresponding VC in the storage flow
    path_index: usize,
}
```

``` rust
pub struct VerifyInput {
    birthdate_threshold: NaiveDate,
    // The merkle root of the storage flow when generating proof, 
    // should be consistent with `ProveInput::merkle_root`
    root: H256,
}
```

### Circuit Constraints

The zero-knowledge proof circuit primarily verifies the following conditions (all must be satisfied):
1. The VC's birthdate is later than the specified `birthdate_threshold` provided in the public inputs.
2. The VC is legitimate data that exists on the storage flow.

**Note:** Although the storage flow supports up to `2^64` leaves, this code only supports VCs located within the first `2^32` leaves, which is equivalent to approximately 1PB of storage.

## Details

### Serialization of Verifiable Certificates

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