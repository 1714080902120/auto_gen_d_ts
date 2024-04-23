mod args;
mod ask;
mod compile;
mod constants;
mod file_io;
use args::InputArgs;
use ask::client;
use file_io::{read::get_files_contents, write::generate_dts_files};
use std::env;

use crate::compile::{parse::parse_js_2_ast, transform::visit_ast, Transformed};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get args
    let args = InputArgs {
        input_path: env::current_dir()?,
        output_path: env::current_dir()?,
        deep: false
    }
    .init()
    .await?;

    // read files content
    let (files, dirs) = get_files_contents(&args.input_path, args.deep).await?;
    println!("-------------一共需要处理 {} 个文件（可能包含空文件）--------------", files.len());
    // compile
    let transformed_files = files
        .into_iter()
        .map(|mut file| {
            let parse_res = parse_js_2_ast(&file.name, &mut file.content);
            let res = visit_ast(parse_res.0, &parse_res.1);
            Transformed {
                file,
                filter_code: res.1,
            }
        })
        .collect::<Vec<Transformed>>();

    // request
    println!("---------------------正在处理中，请耐心等待---------------------");
    let code_blocks = client::batch_ask(transformed_files).await?;
    println!("---------------------处理完毕，共{}个block，准备生成文件---------------------", code_blocks.len());
    
    // write file
    generate_dts_files(&args.output_path, &code_blocks, dirs).await?;
    println!("---------------------生成文件完毕，感谢使用---------------------");
    Ok(())
}

// demo
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let key = env::var(Q_WEN_KEY_NAME)?;
//     let client = Client::new();
//     let url = String::from(
//         "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation",
//     );
//     let file_str = fs::read_to_string("./test/jump.js").await?;
//     let msg = format!(
//     "我有一组JavaScript函数：
//     ```javscript
//     {}
//     ```
//     我不需要你解释，只需要给出类型代码即可，不要你任何的描述，也不要注释。
//     我希望你可以给出这些函数的typescript类型。
//     我只要类型，并且可以放到d.ts文件中作为这些函数的类型提示。
//     ",
//         file_str
//     );
//     let payload = json!({
//         "input": {
//             "prompt": msg
//         },
//         "parameters": {
//             "temperature": 0.9,
//             "top_p": 0.9,
//             "max_tokens": 1024,
//             "stream": false,
//             "result_format": "message",
//         },
//         "model": "qwen-plus",

//     });
//     let res = client
//         .post(url)
//         .header("Authorization", format!("Bearer {}", key))
//         .json(&payload)
//         .send()
//         .await?;
//     let res: serde_json::Value = res.json().await?;
//     let res = res["output"]["choices"][0]["message"]["content"]
//         .as_str()
//         .unwrap();
//     match gen::CodeBlock::find_code_block(res) {
//         Some(code_block) => {
//             dbg!(&code_block);
//             let content = code_block.content;
//             fs::OpenOptions::new()
//                 .write(true)
//                 .create(true)
//                 .truncate(true)
//                 .open(path::Path::new("./test/jump").with_extension("d.ts"))
//                 .await?
//                 .write_all(content.as_bytes())
//                 .await?;
//         }
//         _ => {}
//     };

//     Ok(())
// }
