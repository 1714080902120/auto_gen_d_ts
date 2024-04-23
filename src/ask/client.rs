use std::collections::HashMap;

use crate::{
    compile::{
        gen::{self, CodeBlock},
        Transformed,
    },
    constants::{get_q_wen_key, Q_WEN_TEXT_GEN_API},
};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use tokio::time::{sleep, Duration};

pub fn create_client() -> Client {
    reqwest::Client::new()
}

// #[derive(Debug, Default)]
// struct SendData {
//     state: bool,
//     data: CodeBlock
// }

// 这里只能通过通信的方案去获取response的数据
// 动态错误不是线程安全的，所以这里搞不了动态错误，写起来有点恶心
pub async fn batch_ask(
    files: Vec<Transformed>,
) -> Result<Vec<CodeBlock>, Box<dyn std::error::Error>> {
    let files_len = files.len();
    let (tx, mut rx) = mpsc::channel(100);
    let mut tasks = Vec::with_capacity(files_len);

    // 因为QPS只有20，所以需要限制下
    // 貌似还没有20.。。。日了狗
    // 看来得想个方案
    // 原来是1分钟200次。。。
    // 也不是200啊，骗子
    // 测试下来59就炸了
    // 算了，改成每秒的，200次 / 60 = 3
    // 3秒也不行，1秒2个吧
    let max_requests_per_second = 3;

    let mut num_requests = 0;

    for file in files {
        for funcs in file.filter_code {
            if num_requests >= max_requests_per_second {
                println!("----------超速被抓了，等待一秒-------------");
                sleep(Duration::from_secs_f32(1.5)).await;
                num_requests = 0;
            }
            num_requests += 1;

            let new_tx = tx.clone();
            let file_name = file.file.relative_name.clone();
            let task = tokio::spawn(async move {
                let res = get_qwen_code_block(file_name, funcs).await;
                match res {
                    Ok(r) => match new_tx.send(r).await {
                        Err(e) => {
                            let msg = format!("线程发送失败：{}", e);
                            println!("{}", msg);
                            drop(new_tx);
                        }
                        _ => {
                            drop(new_tx);
                        },
                    },
                    Err(_) => {
                        println!("请求失败");
                        drop(new_tx);
                    }
                };
            });
            tasks.push(task);
        }
    }
    drop(tx);
    let mut code_block_map = HashMap::new();
    
    loop {
        match rx.recv().await {
            Some(message) => {
                code_block_map
                    .entry(message.file_name.clone())
                    .or_insert(vec![])
                    .push(message);
            }
            None => {
                // 数据结构设计的不太行，待优化TODO
                println!("----------请求结束--罚款200，扣两分---------");
                return Ok(code_block_map
                    .values()
                    .map(|v| {
                        let mut code_block = CodeBlock::default();
                        code_block.file_name = v[0].file_name.clone();
                        code_block.language = v[0].language.clone();
                        v.iter().fold(code_block, |mut code_block, curr| {
                            code_block.content += &format!("\n{}", &curr.content);
                            code_block
                        })
                    })
                    .collect());
            }
        }
    }
}

pub async fn get_qwen_code_block(file_name: String, vs: Vec<String>) -> Result<CodeBlock, ()> {
    let key = get_q_wen_key().expect("获取key失败");
    let cli = create_client();
    let payload = gen_pay_load(vs.join("\n"));
    let res = cli
        .post(Q_WEN_TEXT_GEN_API)
        .header("Authorization", format!("Bearer {}", key))
        .json(&payload)
        .send()
        .await
        .expect(&format!("请求失败：{}", &payload));

    let res: Value = res
        .json()
        .await
        .expect(&format!("数据json化失败：{:#?}", &payload));

    let res = res["output"]["choices"][0]["message"]["content"]
        .as_str()
        .expect(&format!("获取字段失败：{:#?}", &res));

    match gen::CodeBlock::find_code_block(file_name, res) {
        Some(code_block) => Ok(code_block),
        _ => {
            let msg = format!("获取code block失败：{:#?}", &res);
            println!("{}", msg);
            Err(())
        }
    }
}

// 直接生成类型的精度太低了，还是得考虑大佬的生成json然后自己组装的方案
fn gen_pay_load(source_code: String) -> Value {
    let promot = format!(
        "作为一名前端工程师的 AI 辅助，我的任务是将提取 JS 源代码中的函数类型描述，生成符合typescript的d.ts类型文件格式的类型
        示例1:
        Q: 源代码如下:
            ```javascript
            {}
            ```
        A: 我返回的结果为：
            ```typescript
            {}
            ```
        源代码如下: {}
        ",
        "
        export function test (a) { return a + 1 }
        export const test2 = () => { return \"haha\" }
        export async function test3 (b: string) { return b }
        export const test4 = async (c: number, d: number) => { return c + d }
        ",
        "
        export declare function test (a: number): number; 
        export declare const test2: () => string;
        export declare function test3 (b: string | undefined): string | undefined; 
        export declare const test4: (c: number, d: number) => number;
        ",
        source_code,
    );
    json!({
        "input": {
            "prompt": promot
        },
        "parameters": {
            "temperature": 0.8,
            "result_format": "message",
        },
        "model": "qwen-plus", // 还是选这个吧，plus的QPS太低了
    })
}

// 我一步一步地分析了这四个函数，第一个函数名为test,具体描述的是接受一个参数`a`，类型是number，返回值是 number。
//         第二个函数函数名为test2, 具体描述的是一个箭头函数，没有参数，返回值是void。
//         第三个函数函数名为test3, 是一个异步函数，具体描述的是接受一个参数`b`，类型是string | undefined，返回值是 string | undefined，虽然是异步函数，但是类型是不需要声明async的。
//         第四个函数函数名为test4, 是一个异步的箭头函数，具体描述的是接受两个参数，第一个参数`c`，类型是number，第二个参数`d`，类型是number，返回值是 number，虽然是异步函数，但是类型是不需要声明async的。
//         所以我返回的结果为:
