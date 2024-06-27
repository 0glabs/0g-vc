pragma circom 2.0.0;

include "./keccak/keccak.circom";
include "./utils.circom";

// Import from @circomlib/circuits
include "bitify.circom";


template HashLeftRight() {
    signal input left[2];
    signal input right[2];
    signal output hash[2];
    
    signal leftSlice[256] <== UnpackHash()(left);
    signal rightSlice[256] <== UnpackHash()(right);
    
    signal hashInput[512] <== ConcatArray(256, 256)(leftSlice, rightSlice);
    signal hashOutput[256] <== Keccak(512, 256)(hashInput);

    hash <== PackHash()(hashOutput);
}

// if s == 0 returns [in[0], in[1]]
// if s == 1 returns [in[1], in[0]]
template DualMux() {
    signal input in[2][2];
    signal input s;
    signal output out[2][2];

    s * (1 - s) === 0;
    out[0][0] <== (in[1][0] - in[0][0]) * s + in[0][0];
    out[0][1] <== (in[1][1] - in[0][1]) * s + in[0][1];
    
    out[1][0] <== (in[0][0] - in[1][0]) * s + in[1][0];
    out[1][1] <== (in[0][1] - in[1][1]) * s + in[1][1];
}

// Verifies that merkle proof is correct for given merkle root and a leaf
// pathIndices input is an array of 0/1 selectors telling whether given pathElement is on the left or right side of merkle path
template MerkleTreeChecker(levels) {
    signal input leafHash[2];
    signal input pathElements[levels][2];
    signal input pathIndices[levels];
    signal output root[2];

    component selectors[levels];
    component hashers[levels];

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== i == 0 ? leafHash : hashers[i - 1].hash;
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = HashLeftRight();
        hashers[i].left <== selectors[i].out[0];
        hashers[i].right <== selectors[i].out[1];
    }

    root <== hashers[levels - 1].hash;
}