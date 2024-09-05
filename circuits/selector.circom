import "circomlib/circuits/comparators.circom";

template Selector(n) {
    signal input index; 
    signal input vec[n];
    signal output selected;

    signal select[n];
    signal selectDiff[n];
    component checkEqual[n];
    component elementProduct[n];
    var lc = 0;
    var slc = 0;

    for (var i = 0; i < n; i++){
        selectDiff[i] <== index - i;
        checkEqual[i] = IsZero();
        checkEqual[i].in <== selectDiff[i];
        elementProduct[i] <== vec[i] * checkEqual[i].out;
        slc += checkEqual[i].out;
        lc += elementProduct[i];
    }

    slc === 1;
    selected <== lc;
}