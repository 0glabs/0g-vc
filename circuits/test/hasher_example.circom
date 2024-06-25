pragma circom 2.0.0;

include "../keccak.circom";
include "../../node_modules/circomlib/circuits/bitify.circom";

template Hasher() {
    var input_len = 128; 

    signal input pre[input_len];
    signal output hash[256];

    signal hasher_input[input_len*8];
    
    // bitfy
    component byteTobits[input_len];
    for (var i = 0; i < input_len; i++) {
        byteTobits[i] = Num2Bits(8);
        byteTobits[i].in <== pre[i];
        for (var j = 0; j < 8; j++) {
            hasher_input[i*8+j] <== byteTobits[i].out[j];
        }
    }

    component hasher = Keccak(input_len*8, 256);
    for (var i = 0; i < input_len*8; i++) {
        hasher.in[i] <== hasher_input[i];
    }
    
    for (var i = 0; i < 256; i++) {
        hash[i] <== hasher.out[i];
    }
}

component main = Hasher();