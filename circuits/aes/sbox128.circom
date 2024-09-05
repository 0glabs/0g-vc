// AES-CTR encryption, modified from crema-labs.
// LICENSE file https://github.com/crema-labs/aes-circom/blob/96666f5072e56e64631af1d21cab1f642c03070c/LICENSE
pragma circom 2.1.9;

include "../aes_ff.circom";
include "circomlib/circuits/comparators.circom";

template SBox128() {
    signal input in;
    signal output out;

    signal inv <== FieldInv()(in);
    signal invBits[8] <== Num2Bits(8)(inv);
    signal outBits[8] <== AffineTransform()(invBits);
    out <== Bits2Num(8)(outBits);
}