pragma circom 2.0.0;

template Multiplier2() {
    signal input in[2];
    signal output out;
    out <== in[0] * in[1];
 }

component main {public [in]} = Multiplier2();