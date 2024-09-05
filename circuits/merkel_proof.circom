pragma circom 2.0.0;

include "./keccak/keccak.circom";
include "./utils.circom";

// Import from @circomlib/circuits
include "circomlib/circuits/bitify.circom";


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
    signal input pathLength;
    signal output root[2];

    signal levelOutput[levels][2];

    component selectors[levels];
    // component hashers[levels];

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== i == 0 ? leafHash : levelOutput[i - 1];
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        levelOutput[i] <== HashLeftRight()(selectors[i].out[0], selectors[i].out[1]);
    }

    root <== SelectArrayElement(levels)(pathLength - 1, levelOutput);
}


template SelectArrayElement(n) {
    assert(n <= 64);
    signal input selector; 
    signal input inputArray[n][2]; 
    signal output out[2]; 

    signal compOutput0 <== LessThan(6)([selector, n]);
    compOutput0 === 1;
    signal compOutput1 <== GreaterEqThan(6)([selector, 0]);
    compOutput1 === 1;
    
    signal intermediates[n + 1][2];
    signal choices[n];

    intermediates[0] <== [0, 0];
    for (var i = 0; i < n; i++) {
        choices[i] <== IsZero()(selector - i);
        intermediates[i + 1][0] <== intermediates[i][0] + inputArray[i][0] * choices[i];
        intermediates[i + 1][1] <== intermediates[i][1] + inputArray[i][1] * choices[i];
    }

    out <== intermediates[n];
}