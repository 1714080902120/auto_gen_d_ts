use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tokio::{fs::File as AsyncFile, io::AsyncWriteExt};

use crate::compile::gen::CodeBlock;

pub async fn generate_dts_files(
    output_path: &Path,
    file_name_contents: &Vec<CodeBlock>,
    dirs: Vec<String>,
) -> io::Result<()> {
    let path: PathBuf = output_path.into();
    create_sub_dir(&path, dirs)?;
    let metadata = tokio::fs::metadata(output_path).await?;
    match (metadata.is_dir(), metadata.is_file()) {
        (true, false) => {
            for block in file_name_contents {
                let dts_file_name = format!("{}.d.ts", block.file_name);
                let dts_file_path = path.join(PathBuf::from(dts_file_name));
                println!("{}", &dts_file_path.display());
                create_or_overwrite_file(&dts_file_path, &block.content).await?;
            }
        }
        (false, true) => {
            let block = file_name_contents
                .first()
                .expect("File name contents list must not be empty");
            let dts_file_name = format!("{}.d.ts", &block.file_name);
            let dts_file_path = output_path.with_file_name(dts_file_name);
            create_or_overwrite_file(&dts_file_path, &block.content).await?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unsupported entry type at path: {}", output_path.display()),
            ))
        }
    };

    Ok(())
}

async fn create_or_overwrite_file(path: &Path, content: &str) -> io::Result<()> {
    let mut file = AsyncFile::create(path).await?;
    file.write_all(content.as_bytes()).await?;
    Ok(())
}

fn create_sub_dir(output_path: &Path, dirs: Vec<String>) -> io::Result<()> {
    for dir in dirs {
        fs::create_dir_all(output_path.join(PathBuf::from(dir)))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_dts_files() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::Builder::new()
            .prefix("test_output")
            .tempdir()
            .unwrap();

        // Define file name contents for testing
        let file_name_contents = vec![
            CodeBlock {
                file_name: "file1".to_string(),
                language: "js".to_string(),
                content: "content1".to_string(),
            },
            CodeBlock {
                file_name: "file2".to_string(),
                language: "js".to_string(),
                content: "content2".to_string(),
            },
        ];

        // Call the function under test
        generate_dts_files(temp_dir.path(), &file_name_contents, vec![])
            .await
            .unwrap();

        // Check if the expected files are created with the correct contents
        let file1_path = temp_dir.path().join("file1.d.ts");
        let file2_path = temp_dir.path().join("file2.d.ts");
        assert!(fs::metadata(&file1_path).is_ok());
        assert!(fs::metadata(&file2_path).is_ok());

        let file1_content = fs::read_to_string(&file1_path).unwrap();
        let file2_content = fs::read_to_string(&file2_path).unwrap();
        assert_eq!("content1", file1_content);
        assert_eq!("content2", file2_content);

        // Clean up the temporary directory
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_gen_absoulte_path() {
        let code_blocks = vec![
            CodeBlock {
                file_name: String::from("tt/astts/dss/jump"),
                language: "typescript".to_string(),
                content: "\nimport { GetterTree, StateTree } from 'vuex';\n\nexport type CommData = {\n    isOem?: boolean;\n    isDebug?: boolean;\n    isPre?: boolean;\n    isLoginO?: boolean;\n    aid?: number;\n    siteId?: number;\n    _allSiteMallDomain?: string;\n    _isChanged?: boolean;\n};\n\ntype VuexStore = {\n    
        $store: {\n        state: StateTree;\n        getters: {\n            ['defData/getUltUpdateUrl']: string;\n            ['commData/_allSiteMallDomain']: string;\n        };\n    };\n};\n\nexport interface MallApp {\n    simplePopupWindow(options: { text: string }, callback: () => void, closeCallback: (isSave: boolean) 
        => void): void;\n    bigSave(callback?: () => void): void;\n}\n\ndeclare const jumpToUpdateSiteVersion: (updateUrl?: string | null) => void;\n\ndeclare const openMallTopManageFrame: (\n    initPage: string,\n    params?: string,\n) => void;\n\ndeclare const openTopManageFrame: (\n    initPage: string,\n    params?: string,\n    force?: boolean,\n    afterJump?: () => void,\n) => void;\n".to_string(),
            },
            CodeBlock {
                file_name: "ttt/lock".to_string(),
                language: "typescript".to_string(),
                content: "\nexport type TriggerLock = {\n    trigger: <T>(callback: () => Promise<T>) => Promise<T>;\n};\n\nexport const LockerWrapper: <T>(\n    locker: TriggerLock,\n    func: (...args: any[]) => Promise<T> | T\n) => (...args: any[]) => Promise<T>;\n".to_string(),
            }
        ];
        let output_path = Path::new("\\\\?\\C:\\Users\\xxx\\Documents\\BaiduSyncdisk\\rust_tools\\rust_auto_gen_d_ts\\testt");
        let res = generate_dts_files(
            output_path,
            &code_blocks,
            vec![
                String::from("tt"),
                String::from("tt/astts"),
                String::from("tt/astts/dss"),
                String::from("ttt"),
            ],
        ).await.unwrap();
    }
}
