pragma circom 2.0.0;

// Import from @circomlib/circuits
include "comparators.circom";
include "bitify.circom";

include "./keccak/keccak.circom";
include "./merkel_proof.circom";
include "./utils.circom";

template DecodeVC() {
    var input_len = 79; 
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

    signal input encoded[input_len];

    signal birthDate[birth_len];

    signal output name[name_len];
    signal output age;
    signal output eduLevel;
    signal output serialNo[serial_len];

    signal output birthDateInt;

    // 将输入信号分割为各个字段
    var name_offset = name_prefix_len;
    name <== ArraySlice(input_len, name_offset, name_len)(encoded);

    var age_offset = name_offset + name_len + age_prefix_len;
    age <== encoded[age_offset];

    var birth_offset = age_offset + age_len + birth_prefix_len;
    birthDate <== ArraySlice(input_len, birth_offset, birth_len)(encoded);

    var edu_offset = birth_offset + birth_len + edu_prefix_len;
    eduLevel <== encoded[edu_offset];

    var serial_offset = edu_offset + edu_len + serial_prefix_len;
    serialNo <== ArraySlice(input_len, serial_offset, serial_len)(encoded);

    birthDateInt <== BytesToInt()(birthDate);
}

template HashVC() {
    var input_len = 79;
    var padded_input_len = 256;

    signal input encoded[input_len];
    signal output leafHash[2];
    
    component leafHasher = Keccak(padded_input_len * 8, 256);
    component byteTobits[input_len];

    for (var i = 0; i < input_len; i++) {
        byteTobits[i] = Num2Bits(8);
        byteTobits[i].in <== encoded[i];
        for (var j = 0; j < 8; j++) {
            leafHasher.in[i * 8 + j] <== byteTobits[i].out[j];
        }
    }

    for (var i = input_len * 8; i < padded_input_len * 8; i++) {
        leafHasher.in[i] <== 0;
    }

    leafHash <== PackHash()(leafHasher.out);
}

template VerifyVC(levels) {
    // 参数：
    var vc_len = 79;

    // 输入信号
    signal input encodedVC[vc_len];
    signal input birthDateThreshold;
    signal input pathElements[levels][2];
    signal input pathIndex;

    // 输出信号
    signal output leafHash[2];
    signal output root[2];

    component decodeVC = DecodeVC();
    decodeVC.encoded <== encodedVC;
    
    component birthDateCheck = LessThan(64);
    birthDateCheck.in[0] <== decodeVC.birthDateInt;
    birthDateCheck.in[1] <== birthDateThreshold;
    birthDateCheck.out === 1;

    // bitfy
    leafHash <== HashVC()(encodedVC);

    signal pathIndices[levels] <== Num2Bits(levels)(pathIndex);
    // merkel proof
    root <== MerkleTreeChecker(levels)(leafHash, pathElements, pathIndices);
}

component main {public [birthDateThreshold]} = VerifyVC(32);