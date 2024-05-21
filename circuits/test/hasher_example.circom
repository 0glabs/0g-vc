pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/sha256/sha256.circom";

template Hasher() {
    // signal input pre;
    signal output hash[256];
    
    component hasher = Sha256(32);
    for (var i = 0; i < 32; i++) {
        hasher.in[i] <== 1;
    }
    
    for (var i = 0; i < 256; i++) {
        hash[i] <== hasher.out[i];
    }
}

component main = Hasher();