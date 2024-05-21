pragma circom 2.0.0;

// include "../node_modules/circomlib/circuits/sha256/sha256.circom";
include "../node_modules/circomlib/circuits/comparators.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "./keccak.circom";
include "./merkel_proof.circom";

template BytesToInt() {
    signal input bytes[8];
    signal output result;

    // 中间信号
    signal accum[9];
    accum[0] <== 0;

    for (var i = 0; i < 8; i++) {
        // 计算 accum[i+1] 的值
        accum[i+1] <== accum[i] + bytes[i] * (1 << (i * 8));
    }
    result <== accum[8];
}

template VerifyVC() {
    // 参数：
    var input_len = 79; 
    var padded_input_len = 256;
    var output_len = 256;
    var name_prefix_len = 4;
    var name_len = 16;
    var age_prefix_len = 3;
    var age_len = 1;
    var birth_prefix_len = 5;
    var birth_len = 8;
    var edu_prefix_len = 3;
    var edu_len = 1;
    var serial_prefix_len = 6;
    var serial_len = 32;
    var levels = 3;

    // 输入信号
    signal input encodedVC[input_len];
    signal input birthDateThreshold;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // 输出信号
    signal output leafHash;
    signal output root;

    // 中间信号
    signal name[name_len];
    signal age;
    signal birthDate[birth_len];
    signal eduLevel;
    signal serialNo[serial_len];
    signal birthDateInt;
    signal hasherInput[input_len * 8];

    // 将输入信号分割为各个字段
    var name_offset = name_prefix_len;
    for (var i = 0; i < name_len; i++) {
        name[i] <== encodedVC[i + name_prefix_len];
    }

    var age_offset = name_offset + name_len + age_prefix_len;
    age <== encodedVC[age_offset];

    var birth_offset = age_offset + age_len + birth_prefix_len;
    for (var i = 0; i < birth_len; i++) {
        birthDate[i] <== encodedVC[birth_offset + i];
    }

    var edu_offset = birth_offset + birth_len + edu_prefix_len;
    eduLevel <== encodedVC[edu_offset];

    var serial_offset = edu_offset + edu_len + serial_prefix_len;
    for (var i = 0; i < serial_len; i++) {
        serialNo[i] <== encodedVC[serial_offset + i];
    }

    // 约束断言: birth_date组成的u64大于给定阈值
    component toInt = BytesToInt();
    for (var i = 0; i < 8; i++) {
        toInt.bytes[i] <== birthDate[i];
    }
    birthDateInt <== toInt.result;
    component birthDateCheck = LessThan(64);
    birthDateCheck.in[0] <== birthDateInt;
    birthDateCheck.in[1] <== birthDateThreshold;
    birthDateCheck.out === 1;

    // bitfy
    component byteTobits[input_len];
    for (var i = 0; i < input_len; i++) {
        byteTobits[i] = Num2Bits(8);
        byteTobits[i].in <== encodedVC[i];
        for (var j = 0; j < 8; j++) {
            hasherInput[i * 8 + j] <== byteTobits[i].out[j];
        }
    }

    // 计算targetHash
    component leafHasher = Keccak(padded_input_len * 8, output_len);
    for (var i = 0; i < input_len * 8; i++) {
        leafHasher.in[i] <== hasherInput[i];
    }
    for (var i = input_len * 8; i < padded_input_len * 8; i++) {
        leafHasher.in[i] <== 0;
    }
    component packer = Bits2Num(output_len);
    for (var i = 0; i < output_len; i++) {
        packer.in[i] <== leafHasher.out[i];
    }
    leafHash <== packer.out;

    // merkel proof
    component mtp = MerkleTreeChecker(levels, output_len);
    mtp.leaf <== leafHash;
    for (var i = 0; i < levels; i++) {
        mtp.pathElements[i] <== pathElements[i];
        mtp.pathIndices[i] <== pathIndices[i];
    }

    root <== mtp.root;
}

component main {public [birthDateThreshold]} = VerifyVC();