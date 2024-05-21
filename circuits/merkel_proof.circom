pragma circom 2.0.0;

include "./keccak.circom";
include "../node_modules/circomlib/circuits/bitify.circom";

// Computes MiMC([left, right])
template HashLeftRight(output_len) {
    signal input left;
    signal input right;
    signal output hash;

    component hasher = Keccak(output_len * 2, output_len);
    component unpacker[2];

    signal leftRight[output_len * 2];
    
    unpacker[0] = Num2Bits(output_len);
    unpacker[0].in <== left;
    unpacker[1] = Num2Bits(output_len);
    unpacker[1].in <== right;
    
    for (var i = 0; i < output_len; i++) {
        leftRight[i] <== unpacker[0].out[i];
        leftRight[output_len + i] <== unpacker[1].out[i];
    }
    
    for (var i = 0; i < output_len * 2; i++) {
        hasher.in[i] <== leftRight[i];
    }

    component packer = Bits2Num(output_len);
    for (var i = 0; i < output_len; i++) {
        packer.in[i] <== hasher.out[i];
    }
    hash <== packer.out;
}

// if s == 0 returns [in[0], in[1]]
// if s == 1 returns [in[1], in[0]]
template DualMux() {
    signal input in[2];
    signal input s;
    signal output out[2];

    s * (1 - s) === 0;
    out[0] <== (in[1] - in[0])*s + in[0];
    out[1] <== (in[0] - in[1])*s + in[1];
}

// Verifies that merkle proof is correct for given merkle root and a leaf
// pathIndices input is an array of 0/1 selectors telling whether given pathElement is on the left or right side of merkle path
template MerkleTreeChecker(levels, output_len) {
    signal input leaf;
    signal input pathElements[levels];
    signal input pathIndices[levels];
    signal output root;

    component selectors[levels];
    component hashers[levels];

    for (var i = 0; i < levels; i++) {
        selectors[i] = DualMux();
        selectors[i].in[0] <== i == 0 ? leaf : hashers[i - 1].hash;
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = HashLeftRight(output_len);
        hashers[i].left <== selectors[i].out[0];
        hashers[i].right <== selectors[i].out[1];
    }

    root <== hashers[levels - 1].hash;
}