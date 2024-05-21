# 背景
提供 ZK 工具，允许0G用户使用 ZK 工具对可验证证书的字段进行任意粒度的展示、证明。
例如，用户在0G上存有包括：姓名，年龄，出生日期，学历代号等字段的可验证证书，用户希望向他人展示其是00后。
# 设计
## 证书格式
可验证证书至少应包含一个serial_no字段，用于确定证书的所有权，知晓serial_no的用户持有该证书。证书的其他字段因其是什么证书存在差异。serial_no的另一作用是防止穷举推测隐私字段。可定制。这里以学历证书为例对证书格式做说明。
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
education为学历代号，如3代表硕士研究生。证书各字段应做合适编码，方便后续哈希，编码后的VC格式如下：
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
- 随序列号：前缀为"serial", 主体部分32字节。

编码后VC有效数据部分为79字节，padding到256字节。
![Screenshot 2024-05-21 at 14.53.23.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716274410323-b8a8e4fd-f9c2-4e48-9336-e34f43ee3468.png#averageHue=%23ededed&clientId=u4e87bca3-f673-4&from=drop&id=ub57c7a25&originHeight=286&originWidth=2344&originalType=binary&ratio=2&rotation=0&showTitle=false&size=133204&status=done&style=none&taskId=ude550725-cda6-40f7-b45d-11e6a6e2a3d&title=)
EVC经过哈希得到VC tree的叶子leaf，多个VC向上计算得到根哈希root。这里使用的哈希函数为Keccak。EVC到leaf的计算采用Keccak(256*8, 256)，中间节点计算采用Keccak(256*2, 256)。
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
项目目录如下，其中：
![Screenshot 2024-05-21 at 15.27.24.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716276450567-4af591fd-bdd0-453a-9055-a2da39ae3611.png#averageHue=%23222222&clientId=u4e87bca3-f673-4&from=drop&height=264&id=w2B6T&originHeight=668&originWidth=382&originalType=binary&ratio=2&rotation=0&showTitle=false&size=71638&status=done&style=none&taskId=ue3ac2436-99f1-4814-967f-3d367829a99&title=&width=151)

- circuits为circom电路，主要是VC电路、Keccak哈希电路、Merkel proof电路以及一些公用的工具电路。_注意这里Keccak哈希电路做了一些改造，以接受任意长度的input，原来的inputs长度不能超过blocksize=136bytes，不符合我们LeafHasher的要求（输入长度为256bytes），这里可能导致电路中算出的哈希和外部算出的不一致。_
- fronted脚本，是对circom电路的编译逻辑，用到了circom compiler。
- circomlib电路库，是官方提供的一些库电路。
- output，fronted部分的输出保存在该目录下，主要是生成的r1cs文件和witness calculator文件。
- src是主要的代码逻辑，这部分分为证书格式化、证明前端和证明后端三个部
   - 证书格式化负责将用户可读的证书格式编码为证明的输入格式

![Screenshot 2024-05-09 at 11.30.11.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1715225416624-cd4eccac-b57f-4259-9f2f-c4d0a00d2fd3.png#averageHue=%23202020&clientId=u0ea9e388-7d30-4&from=drop&height=265&id=IR3jk&originHeight=802&originWidth=566&originalType=binary&ratio=2&rotation=0&showTitle=false&size=104145&status=done&style=none&taskId=u41eabfd8-725c-4229-9eaa-9df3b302d0f&title=&width=187)

   - 证明前端使用circom compiler，实际调用了frontend脚本，将输入的.circom文件编译为证明后端输入r1cs以及生成witness计算代码。

![Screenshot 2024-05-09 at 11.26.46.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1715225211681-58413ad2-26db-4ea1-9f99-8ce7aa10a45d.png#averageHue=%23212120&clientId=u0ea9e388-7d30-4&from=drop&id=DFu7U&originHeight=638&originWidth=1472&originalType=binary&ratio=2&rotation=0&showTitle=false&size=177097&status=done&style=none&taskId=ufbabe0be-9c53-4ac0-ba19-09edd8bb94c&title=)

   - 证明后端，
      - 计算witness

![Screenshot 2024-05-09 at 11.27.22.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1715225247798-3ae3a8b7-b18f-416c-b905-619f79f19b0d.png#averageHue=%23212121&clientId=u0ea9e388-7d30-4&from=drop&id=b7UeQ&originHeight=764&originWidth=1404&originalType=binary&ratio=2&rotation=0&showTitle=false&size=201009&status=done&style=none&taskId=u59ca33df-76b5-4125-9f29-6d8a3426f8f&title=)

      - 使用对应的后端系统生成、验证证明

![Screenshot 2024-05-09 at 11.27.55.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1715225280505-dc10d153-ed48-4ec8-8ebc-6c76445169c6.png#averageHue=%23202020&clientId=u0ea9e388-7d30-4&from=drop&id=YUEta&originHeight=450&originWidth=1490&originalType=binary&ratio=2&rotation=0&showTitle=false&size=125116&status=done&style=none&taskId=u2652a4b9-0731-4ee5-9e1e-f706f1eac2a&title=)
## 如何使用？

- 执行文件

项目提供了实例文件src/main.rs，给出了一个证明证书的birth_data字段小于2000年3月4日的证明。需_要注意的是这里假设0G上存储的VC tree如上面**证书格式**章节的3层VC tree所示，仅有一个VC存储，其他均为空。_
![Screenshot 2024-05-21 at 16.11.05.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716279070689-fd4213bf-3687-4ee2-9b87-d46d1baa7309.png#averageHue=%23212121&clientId=u4e87bca3-f673-4&from=drop&id=u0e839476&originHeight=1412&originWidth=1094&originalType=binary&ratio=2&rotation=0&showTitle=false&size=336891&status=done&style=none&taskId=uf8657e54-fbe1-407e-8730-fa69e377041&title=)

- 执行
```shell
cargo build --release
./target/release/vc-prove
```

- 执行结果

![Screenshot 2024-05-21 at 16.17.52.png](https://cdn.nlark.com/yuque/0/2024/png/2564997/1716279477603-cf1932ff-9cf0-46a5-966d-489a5e8ed514.png#averageHue=%23010100&clientId=u4e87bca3-f673-4&from=drop&id=udcb0fb02&originHeight=734&originWidth=3022&originalType=binary&ratio=2&rotation=0&showTitle=false&size=855655&status=done&style=none&taskId=u435678ac-d720-4cbe-886d-5208184de22&title=)
