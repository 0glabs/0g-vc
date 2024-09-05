// AES-CTR encryption, modified from crema-labs.
// LICENSE file https://github.com/crema-labs/aes-circom/blob/96666f5072e56e64631af1d21cab1f642c03070c/LICENSE
pragma circom 2.1.9;

include "transformations.circom";
include "../aes_ff.circom";
include "circomlib/circuits/bitify.circom";

//tbox[0] =>> multiplication by 2
//tbox[1] =>> multiplication by 3
template TBox(index) {
    signal input in;
    signal output out;

    if (index == 0) {
        out <== FieldMul2()(in);
    } else if (index == 1) {
        out <== FieldMul3()(in);
    }
}