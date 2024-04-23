use std::fs::{canonicalize, File, OpenOptions};
use std::io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};
#[derive(Debug)]
pub struct InputArgs {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub deep: bool,
}

impl InputArgs {
    pub async fn init(&self) -> Result<Self, Box<dyn std::error::Error>> {
        self.absoluted_path(self.get_input().await?).await
    }
    async fn get_input(&self) -> Result<(String, String, bool), Box<dyn std::error::Error>> {
        print!("请输入目标文件（夹）地址: ");

        let mut input_file_path = String::new();

        get_path_from_command_line(&mut input_file_path)?;

        print!("请输入输出文件夹地址（注意是文件夹）: ");

        let mut output_file_path = String::new();

        get_path_from_command_line(&mut output_file_path)?;

        print!("要深度♂吗？即递归子文件夹（y/n）: ");

        let mut need_deep = String::new();
        get_path_from_command_line(&mut need_deep)?;
        Ok((
            input_file_path,
            output_file_path,
            need_deep.replace("\r\n", "").to_lowercase().eq("y"),
        ))
    }
    async fn absoluted_path(
        &self,
        input_args: (String, String, bool),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (input_file_path, output_file_path, deep) = input_args;
        let input_path =
            trans_2_absolute(&input_file_path.replace("\r", "").replace("\n", ""), true)?;
        let output_path =
            trans_2_absolute(&output_file_path.replace("\r", "").replace("\n", ""), false)?;
        Ok(Self {
            input_path,
            output_path,
            deep,
        })
    }
}

fn get_path_from_command_line(path: &mut String) -> Result<(), std::io::Error> {
    stdout().flush()?;
    stdin().read_line(path)?;

    path.trim_end_matches('\n');
    Ok(())
}

fn trans_2_absolute(path: &str, is_input: bool) -> Result<PathBuf, std::io::Error> {
    let is_absolute = Path::new(&path).is_absolute();
    match is_absolute {
        false => match canonicalize(path) {
            Ok(path) => Ok(path),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if is_input {
                        println!("找不到目标文件或者文件夹：{}", &path);
                        return Err(e);
                    }
                    // 创建文件夹
                    let msg = format!("文件路径找不到，准备创建文件夹：{}", &path);
                    println!("{}", msg);
                    std::fs::create_dir_all(&path)?;
                    let target_path = canonicalize(&path).expect("获取绝对路径失败");
                    Ok(target_path)
                }
                _ => Err(e),
            },
        },
        _ => {
            let target_path = PathBuf::from(&path);

            if !target_path.exists() {
                if is_input {
                    println!("找不到目标文件或者文件夹：{}", &path);
                    return Err(io::Error::new(io::ErrorKind::NotFound, "啧啧啧"));
                }
                println!("路径并不存在：{}", &path);
                println!("正在尝试创建文件夹");
                match fs::create_dir_all(&target_path) {
                    Ok(_) => {
                        println!("创建成功：{}", &path);
                    }
                    Err(e) => {
                        println!("创建失败：{}", e);
                        return Err(e);
                    }
                }
            }
            Ok(PathBuf::from(path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tokio::{fs, io::AsyncWriteExt};

    #[tokio::test]
    async fn test_absoulte_path_not_exist() {
        let input_path = r"C:\商城项目\mallapp-designer\v3\utils";
        let output_path = r"C:\Users\liudanrui\Documents\BaiduSyncdisk\rust_tools\rust_auto_gen_d_ts\testt\tt\t.js";

        let result = trans_2_absolute(input_path, true).expect("get path error");
        dbg!(result);
        let result = trans_2_absolute(output_path, false).expect("get path error");
        dbg!(result);
    }

    #[tokio::test]
    async fn test_relative_path_not_exist() {
        let input_path = r"C:\商城项目\mallapp-designer\v3\utils";
        let output_path = r"../rust_auto_gen_d_ts/testt/tt";

        let result = trans_2_absolute(input_path, true).expect("get path error");
        dbg!(result);
        let result = trans_2_absolute(output_path, false).expect("get path error");
        dbg!(result);
    }

    #[tokio::test]
    async fn test_relative_path_exist() {
        let input_path = r"C:\商城项目\mallapp-designer\v3\utils";
        let output_path = r"../rust_auto_gen_d_ts/testt/";

        let result = trans_2_absolute(input_path, true).expect("get path error");
        dbg!(result);
        let result = trans_2_absolute(output_path, false).expect("get path error");
        dbg!(result);
    }

    #[tokio::test]
    async fn test_init() {
        // 创建临时文件夹和文件
        let temp_dir = fs::create_dir_all("/tmp/test_dir").await.unwrap();
        let mut temp_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("/tmp/test_dir/test_file.txt")
            .await
            .unwrap();
        let _ = temp_file.write_all(b"test input data").await.unwrap();

        // 设置输入参数
        let input_path = Path::new("/tmp/test_dir/test_file.txt");
        let output_path = Path::new("/tmp/test_dir/test_output.txt");
        let input_args = InputArgs {
            input_path: input_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            deep: true,
        };

        // 执行初始化函数
        let result = input_args.init().await.unwrap();

        // 验证结果是否正确
        assert_eq!(result.input_path, input_path.to_path_buf());
        assert_eq!(result.output_path, output_path.to_path_buf());
    }

    use std::io;
    use std::path::PathBuf;

    #[test]
    fn test_trans_2_absolute_with_absolute_windows_path() {
        let path = r#"C:\Users\user\Documents"#;
        let result = trans_2_absolute(path, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from(path));
    }

    #[test]
    fn test_trans_2_absolute_with_relative_windows_path() {
        let path = r#"C:\商城项目\mallapp-designer\v3\utils"#;
        dbg!(PathBuf::from(path));
        let result = trans_2_absolute(path, false);
        assert!(result.is_ok());
        // Assert the canonicalized path is correct
        let canonical_path = std::fs::canonicalize(path).unwrap();
        assert_eq!(result.unwrap(), canonical_path);
    }

    #[test]
    fn test_trans_2_absolute_with_invalid_windows_path() {
        let path = r#"./test"#;
        let result = trans_2_absolute(path, false);
        dbg!(&result);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotFound);
    }
}
