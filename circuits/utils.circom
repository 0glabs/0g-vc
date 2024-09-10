pragma circom 2.0.0;

template BytesToInt() {
    signal input bytes[8];
    signal output result;

    var lc = 0;

    for (var i = 0; i < 8; i++) {
        lc += bytes[i] * (1 << (i * 8));
    }
    result <== lc;
}

template BytesToBits(nBytes) {
    signal input bytes[nBytes];
    signal output bits[nBytes * 8];
    component toBits[nBytes];

    for (var i = 0; i < nBytes; i++) {
        toBits[i] = Num2Bits(8);
        toBits[i].in <== bytes[i];
        for (var j = 0; j < 8; j++) {
            bits[i * 8 + j] <== toBits[i].out[j];
        }
    }
}

template BitsToBytes(nBytes) {
    signal input bits[nBytes * 8];
    signal output bytes[nBytes];
    component toBytes[nBytes];

    for (var i = 0; i < nBytes; i++) {
        toBytes[i] = Bits2Num(8);
        for (var j = 0; j < 8; j++) {
            toBytes[i].in[j] <== bits[i * 8 + j];
        }
        bytes[i] <== toBytes[i].out;
    }
}

template ArraySlice(N, START, LENGTH) {
    signal input in[N];
    signal output out[LENGTH];

    for (var i = 0; i < LENGTH; i++) {
        out[i] <== in[START + i];
    }
}

template ConcatArray(N, M) {
    signal input in1[N], in2[M];
    signal output out[N + M];

    for (var i = 0; i < N; i++) {
        out[i] <== in1[i];
    }

    for (var i = 0; i < M; i++) {
        out[N + i] <== in2[i];
    }
}

template PadZero(N, M) {
    assert(N <= M);
    signal input in[N];
    signal output out[M];

    for (var i = 0; i < N; i++) {
        out[i] <== in[i];
    }

    for (var i = N; i < M; i++) {
        out[i] <== 0;
    }
}

template PackHash() {
    signal input in[256];
    signal output out[2];

    out[0] <== Bits2Num(128)(ArraySlice(256, 0, 128)(in));
    out[1] <== Bits2Num(128)(ArraySlice(256, 128, 128)(in));
}

template UnpackHash() {
    signal input in[2];
    signal output out[256];

    out <== ConcatArray(128, 128)(Num2Bits(128)(in[0]), Num2Bits(128)(in[1]));
}