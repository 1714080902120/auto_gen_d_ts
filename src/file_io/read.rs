use std::path::Path;
use tokio::{
    fs::{self, ReadDir},
    io::{self, AsyncRead, AsyncReadExt, ReadBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct File {
    pub name: String,
    pub content: String,
    pub relative_name: String,
}

// 要递归处理下了，毕竟可能文件夹里还有文件夹
// 怎么这么讨厌呢
pub async fn get_files_contents(path: &Path, deep: bool) -> io::Result<(Vec<File>, Vec<String>)> {
    let mut contents = vec![];
    let mut dirs = vec![];

    fn recur_dir(
        prefix: String,
        deep: bool,
        path: &Path,
        dirs: &mut Vec<String>,
        contents: &mut Vec<File>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            if !prefix.is_empty() {
                dirs.push(prefix.clone());
            }
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();
                let entry = entry.file_type()?;
                let mut name = file_path
                    .file_name()
                    .expect(&format!("获取文件名失败: {}", file_path.display()))
                    .to_string_lossy()
                    .to_string();

                // 过滤掉测试用例文件夹
                if name.contains("__test__") {
                    continue;
                }

                if entry.is_file() {
                    // 只针对js吧。。
                    if name.ends_with(".js") {
                        name = name.replace(".js", "");
                        let content = std::fs::read_to_string(file_path)?;
                        contents.push(File {
                            relative_name: format!(
                                "{}{}{}",
                                &prefix,
                                if prefix.is_empty() { "" } else { "/" },
                                name
                            ),
                            name,
                            content,
                        });
                    }
                } else if entry.is_dir() {
                    if !deep {
                        continue;
                    } else {
                        recur_dir(
                            format!(
                                "{}{}{}",
                                prefix,
                                if prefix.is_empty() { "" } else { "/" },
                                name
                            ),
                            deep,
                            &file_path,
                            dirs,
                            contents,
                        )?;
                    }
                }
            }
            Ok(())
        } else if metadata.is_file() {
            let mut name = path
                .file_name()
                .expect(&format!("获取文件名失败: {}", &path.display()))
                .to_string_lossy()
                .to_string();
            // 只针对js吧。。
            if name.ends_with(".js") {
                let content = std::fs::read_to_string(path)?;
                name = name.replace(".js", "");
                contents.push(File {
                    relative_name: format!(
                        "{}{}{}",
                        &prefix,
                        if prefix.is_empty() { "" } else { "/" },
                        name
                    ),
                    name,
                    content,
                });

                Ok(())
            } else {
                Err(format!("非js文件: {}", path.display()).into())
            }
        } else {
            Err(format!("你这是什么路径？！: {}", path.display()).into())
        }
    }

    recur_dir(String::new(), deep, path, &mut dirs, &mut contents);
    Ok((contents, dirs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::path;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_files_and_contents() -> io::Result<()> {
        // Create a temporary directory with some files
        let temp_dir = TempDir::new()?;
        let temp_dir_path = temp_dir.path();
        let file1_path = temp_dir_path.join("file1.txt");
        let file2_path = temp_dir_path.join("file2.txt");
        fs::write(file1_path, "content1")?;
        fs::write(file2_path, "content2")?;

        let files_with_contents = get_files_contents(temp_dir_path, true).await?;

        dbg!(files_with_contents);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_files_contents() -> io::Result<()> {
        let files = get_files_contents(path::Path::new("./test"), true).await?;
        dbg!(files);
        Ok(())
    }
}
