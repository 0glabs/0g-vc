pragma circom 2.0.0;

// Import from @circomlib/circuits
include "circomlib/circuits/comparators.circom";
include "circomlib/circuits/bitify.circom";

include "./keccak/keccak.circom";
include "./merkel_proof.circom";
include "./utils.circom";
include "./custom.circom";
include "./aes/ctr.circom";

function VcLen() {
    return 79;
}

template DecodeVC() {
    var input_len = VcLen(); 
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
    var input_len = VcLen();
    var hash_len = 32;
    var leaf_len = 256;
    var encrypt_len = input_len + hash_len;
    assert(16 + encrypt_len <= leaf_len);

    signal input encoded[input_len];
    signal input aesKey[16];
    signal input aesIV[16];
    signal output leafHash[2];
    
    component vcHasher = Keccak(input_len * 8, hash_len * 8);
    vcHasher.in <== BytesToBits(input_len)(encoded);
    signal vcHashBytes[hash_len] <== BitsToBytes(hash_len)(vcHasher.out);

    signal plainText[encrypt_len] <== ConcatArray(input_len, hash_len)(encoded, vcHashBytes);
    signal cipherText[encrypt_len] <== EncryptCTR(encrypt_len, 4)(plainText, aesIV, aesKey);

    signal leafContent[16 + encrypt_len] <== ConcatArray(16, encrypt_len)(aesIV, cipherText);
    signal paddedLeaf[leaf_len] <== PadZero(16 + encrypt_len, leaf_len)(leafContent);

    component leafHasher = Keccak(leaf_len * 8, hash_len * 8);
    leafHasher.in <== BytesToBits(leaf_len)(paddedLeaf);
    leafHash <== PackHash()(leafHasher.out);
}

template VerifyVC(levels) {
    // 参数：
    var vc_len = VcLen();
    var num_extensions = 16;

    // 输入信号
    signal input encodedVC[vc_len];
    signal input aesKey[16];
    signal input aesIV[16];
    signal input extensions[num_extensions];
    signal input pathElements[levels][2];
    signal input pathIndex;
    signal input pathLength;

    // 输出信号
    signal output root[2];

    component decodeVC = DecodeVC();
    decodeVC.encoded <== encodedVC;
    
    CustomCheck()(decodeVC.name, decodeVC.age, decodeVC.eduLevel, decodeVC.serialNo, decodeVC.birthDateInt, extensions);
    
    // merkel proof
    signal pathIndices[levels] <== Num2Bits(levels)(pathIndex);
    signal leafHash[2] <== HashVC()(encodedVC, aesKey, aesIV);
    root <== MerkleTreeChecker(levels)(leafHash, pathElements, pathIndices, pathLength);
}

component main {public [extensions]} = VerifyVC(40);