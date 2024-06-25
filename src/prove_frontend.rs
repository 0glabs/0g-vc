use std::env;
use std::process::Command;

#[allow(dead_code)]
pub fn compile_circuit(input_file: &str, output_dir: &str) {
    // 指定要执行的 Shell 文件路径
    let project_dir = env::current_dir().expect("Failed to get current directory");
    let script_path = project_dir.join("frontend.sh");

    // 使用 Command::new 创建一个新的命令
    let mut cmd = Command::new("sh");

    // 将要执行的 Shell 文件路径和参数传递给 sh 命令
    cmd.arg(script_path).arg(input_file).arg(output_dir);

    // 执行命令
    let output = cmd.output().expect("Failed to execute shell script");

    // 打印命令的输出
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

#[cfg(test)]
mod test {}
