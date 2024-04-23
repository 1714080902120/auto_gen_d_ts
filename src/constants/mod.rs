use std::env;

pub const Q_WEN_KEY_NAME: &'static str = "Q_WEN_KEY";
pub const Q_WEN_TEXT_GEN_API: &'static str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";
pub const RES_TEXT_RANGE_REG: &'static str =
    r"`{3}(?P<language>\w+)(\n|\r\n?)(?P<content>[\s\S]*?)`{3}";

// 有点危险
pub const FUNCTION_CHUNK_MAX_SIZE: usize = 1024 * 8;


pub fn get_q_wen_key() -> Result<String, std::env::VarError> {
    env::var(Q_WEN_KEY_NAME)
}
