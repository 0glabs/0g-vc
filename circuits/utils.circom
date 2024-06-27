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