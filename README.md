# 背景
提供 ZK 工具，允许0G用户使用 ZK 工具对可验证证书的字段进行任意粒度的展示、证明。
例如，用户在0G上存有包括：姓名，年龄，出生日期，学历代号等字段的可验证证书，用户希望向他人展示其是00后。
# 设计
## 证书格式
可验证证书至少应包含一个serial_no字段，用于确定证书的所有权，知晓serial_no的用户持有该证书。证书的其他字段因其是什么证书存在差异，可定制。serial_no的另一作用是防止穷举推测隐私字段。这里以学历证书为例对证书格式做说明。
学历证书包含姓名，年龄，出生日期，学历代号字段：
```rust
pub struct VC {
    name: String,
    age: u8,
    birth_date: String,
    edu_level: u8,
    serial_no: String,
}
```
证书各字段应做合适编码，方便后续哈希，编码后的VC格式如下：
```rust
pub struct EncodedVC {
    name: Vec<u8>, // 4+16bytes
    age: Vec<u8>, // 3+1bytes
    birth_date: Vec<u8>, // 5+8bytes
    edu_level: Vec<u8>, // 3+1bytes
    serial_no: Vec<u8>, // 5+32bytes
}
```

- name：前缀为"name"，主体部分使用UTF-8编码，长度16字节。
- 年龄：前缀为"age", 主体部分使用u8类型。
- 出生日期：前缀为"birth", 主体部分使用Unix时间戳表示，长度为8字节（u64）。
- 学历代号：前缀为"edu", 主体部分使用u8类型。
- 随序列号：前缀为"serial", 主体部分使用hexString类型，32字节。

编码后VC有效数据部分为79字节，padding到256字节。
![Screenshot 2024-05-21 at 14.53.23.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716274410323-b8a8e4fd-f9c2-4e48-9336-e34f43ee3468.png#averageHue=%23ededed&clientId=u4e87bca3-f673-4&from=drop&id=ub57c7a25&originHeight=286&originWidth=2344&originalType=binary&ratio=2&rotation=0&showTitle=false&size=133204&status=done&style=none&taskId=ude550725-cda6-40f7-b45d-11e6a6e2a3d&title=)
EVC(Encoded VC)经过哈希得到VC tree的叶子leaf，多个VC向上计算得到根哈希root。这里使用的哈希函数为Keccak。EVC到leaf的计算采用Keccak(256*8, 256)，中间节点计算采用Keccak(256*2, 256)。
![Screenshot 2024-05-21 at 15.14.43.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716275690140-ed2a21fa-24e5-4950-af8e-569291af3fd3.png#averageHue=%23f4f4f4&clientId=u4e87bca3-f673-4&from=drop&id=ue90c49cb&originHeight=1024&originWidth=1652&originalType=binary&ratio=2&rotation=0&showTitle=false&size=271073&status=done&style=none&taskId=u5071e7dd-9ca0-4985-a2de-81f78e32398&title=)
## 电路约束
VC tree的root哈希存储在0G上，用户想要证明字段属性（如00后出生）时通过证明：

1. birth_date字段大于(编码后的)20000101
2. VC tree root哈希等于0G上存储的哈希

以上2个断言均成立即可。
# 实践
## 项目依赖

- circom compiler：项目前端电路编译部分依赖于circom compiler，安装过程嵌在frontend.sh脚本中。
- circomlib：项目电路构建于circomlib提供的一些库电路。在项目目录中执行`npm init`命令初始化一个新的 Node.js 项目。之后执行`npm install circomlib`安装电路库。
## 项目结构
项目目录如下

![Screenshot 2024-05-21 at 15.27.24.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716276450567-4af591fd-bdd0-453a-9055-a2da39ae3611.png#averageHue=%23222222&clientId=u4e87bca3-f673-4&from=drop&height=264&id=w2B6T&originHeight=668&originWidth=382&originalType=binary&ratio=2&rotation=0&showTitle=false&size=71638&status=done&style=none&taskId=ue3ac2436-99f1-4814-967f-3d367829a99&title=&width=151)

其中：
- circuits为circom电路，主要是VC电路、Keccak哈希电路、Merkel proof电路以及一些公用的工具电路。_注意这里Keccak哈希电路做了一些改造，以接受任意长度的input，原来的inputs长度不能超过blocksize=136bytes，不符合我们LeafHasher的要求（输入长度为256bytes），这里可能导致电路中算出的哈希和外部算出的不一致。_
- fronted脚本，是对circom电路的编译逻辑，用到了circom compiler。
- circomlib电路库，是官方提供的一些库电路。
- output，frontend部分的输出保存在该目录下，主要是生成的r1cs文件和witness calculator文件。
- src是主要的代码逻辑，这部分分为证书格式化、证明前端和证明后端三个部份：
   - 证书格式化负责将用户可读的证书格式编码为证明输入格式。

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
- - 证明前端使用circom compiler，实际调用了frontend脚本，将输入的.circom文件编译为证明后端输入r1cs以及生成witness计算代码。

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

- - 证明后端，
      - 计算witness

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

   - - - 使用对应的后端系统生成、验证证明

```rust
pub fn gen_proof(
    circuit: CircomCircuit<Bn254>,
    pk: &ProvingKey<Bn254>,
    rng: &mut impl Rng,
) -> Proof<Bn254> {
    GrothBn::create_random_proof_with_reduction(circuit, pk, rng).unwrap()
}

pub fn ver_proof(pk: &ProvingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &Vec<Fr>) -> bool {
    let pvk = prepare_verifying_key(&pk.vk);
    GrothBn::verify_proof(&pvk, proof, public_inputs).unwrap()
}
```
## 如何使用？

- 执行文件

项目提供了示例文件`src/main.rs`，给出了一个证明证书的birth_data字段小于2000年3月4日的证明。_需要注意的是这里假设0G上存储的VC tree如上面**证书格式**章节的3层VC tree所示，仅有一个VC存储，其他均为空。
```rust
fn main() {
    // 1. 解析VC Json并计算编码和哈希
    let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
    let vc: VC = serde_json::from_str(vc_json).unwrap();
    // 1.1. 计算vc编码和哈希
    let (encoded_vc, hash) = vc.hash();
    let circuit_input = encoded_vc.join();
    // 1.2. 计算birthDateThreshold的编码
    let encoded_birth_date = vc.birth_date();
    println!(
        "encoded VC: {:?}, encoded_birth_date: {}",
        encoded_vc,
        // hex::encode(hash),
        encoded_birth_date
    );

    let birth_date_threshold = NaiveDate::parse_from_str("20000304", "%Y%m%d")
        .expect("Invalid birth date string")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp() as u64;
    println!("birth_date_threshold: {}", birth_date_threshold);

    // 2. 编译电路
    compile_circuit("./circuits/check_vc.circom", "output");

    // 3. 计算witness
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let mut inputs = HashMap::new();
    inputs.insert(
        "encodedVC".to_string(),
        circuit_input.iter().map(|&x| BigInt::from(x)).collect(),
    );
    inputs.insert(
        "birthDateThreshold".to_string(),
        vec![BigInt::from(birth_date_threshold)],
    );
    inputs.insert(
        "pathElements".to_string(),
        vec![BigInt::from(0), BigInt::from(0), BigInt::from(0)],
    );
    inputs.insert(
        "pathIndices".to_string(),
        vec![BigInt::from(0), BigInt::from(0), BigInt::from(0)],
    );

    let (circuit, pub_in) = cal_witness(
        current_dir.join("output/check_vc_js/check_vc.wasm"),
        current_dir.join("output/check_vc.r1cs"),
        inputs,
    )
    .unwrap();

    // 3. 生成证明
    let mut rng = thread_rng();
    let params =
        GrothBn::generate_random_parameters_with_reduction(circuit.clone(), &mut rng).unwrap();
    let proof = gen_proof(circuit, &params, &mut rng);

    println!("Proof generated: {:?}", proof);

    // 4. 验证证明
    let result = ver_proof(&params, &proof, &pub_in);
    assert!(result == true);
}
```

- - 执行
```shell
cargo build --release
./target/release/vc-prove
```

- - 执行结果

![Screenshot 2024-05-21 at 16.17.52.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716279477603-cf1932ff-9cf0-46a5-966d-489a5e8ed514.png#averageHue=%23010100&clientId=u4e87bca3-f673-4&from=drop&id=udcb0fb02&originHeight=734&originWidth=3022&originalType=binary&ratio=2&rotation=0&showTitle=false&size=855655&status=done&style=none&taskId=u435678ac-d720-4cbe-886d-5208184de22&title=)
