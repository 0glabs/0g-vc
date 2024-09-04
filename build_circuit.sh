#!/bin/bash

function install_circom() {
    # 检查是否安装了circom
    if [ ! -x "$(command -v circom)" ]; then
        # 如果未安装,使用cargo install安装
        echo "circom未安装,正在使用cargo install安装..."
        cargo install --git https://github.com/iden3/circom.git --rev 2eaaa6d --bin circom
        if [ $? -ne 0 ]; then
            echo "circom安装失败,请手动安装后重试。"
            exit 1
        fi
        echo "circom安装成功。"
    else
        # 如果已安装,输出当前circom版本
        echo "circom已安装,当前版本:"
        circom --version
    fi
}

function build_circuit() {
    # 接受输入的.circom文件和输出路径
    input_file="$1"
    output_dir="$2"

    mkdir -p $output_dir

    # 检查输入的.circom文件是否存在
    if [ ! -f "$input_file" ]; then
        echo "输入的.circom文件不存在,请检查路径后重试"
        exit 1
    fi

    # 获取当前目录
    current_dir=$(pwd)
    # 拼接 node_modules 路径
    circomlib_path="$current_dir/node_modules/circomlib/circuits"

    # 使用circom编译.circom文件,生成r1cs文件
    circom "$input_file" --O2 -l ./node_modules/circomlib/circuits --r1cs --wasm --output "$output_dir" 

    if [ $? -eq 0 ]; then
        echo "r1cs文件生成成功,输出目录: $output_dir"
    else
        echo "r1cs文件生成失败,请检查.circom文件后重试"
        exit 1
    fi
}

# 获取文件所在的目录路径
dir_path=$(dirname "$0")

# 切换到文件所在的目录
cd "$dir_path" || exit
mkdir -p ./output

if [[ $1 == '--install' ]]; then
    install_circom
    exit 0
fi

if [ ! -x "$(command -v circom)" ]; then
    install_circom
fi

if [[ $1 == "custom" ]]; then
    temp_dir=$(mktemp -d --suffix _custom_vc)
    # trap "rm -rf $temp_dir" EXIT
    echo "定制逻辑构建目录: $temp_dir"

    cp -r circuits/* $temp_dir
    cp customized/$2.circom $temp_dir/custom.circom
    mv $temp_dir/check_vc.circom $temp_dir/$2.circom 
    build_circuit "$temp_dir/$2.circom" output
else
    build_circuit "./circuits/check_vc.circom" output
fi
