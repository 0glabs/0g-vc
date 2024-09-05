pragma circom 2.0.0;

include "aes/ctr.circom";
include "aes/sbox128.circom";

component main = EncryptCTR(128, 4);