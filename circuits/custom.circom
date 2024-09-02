pragma circom 2.0.0;

template CustomCheck() {
    var name_len = 16;
    var serial_len = 32;
    var num_extensions = 16;

    signal input name[name_len];
    signal input age;
    signal input eduLevel;
    signal input serialNo[serial_len];
    signal input birthDateInt;

    signal input extensions[num_extensions];

    // Check birthday threshold
    signal birthdayOutput <== LessThan(64)([birthDateInt, extensions[0]]);
    birthdayOutput === 1;
}