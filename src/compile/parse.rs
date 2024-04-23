use std::path::PathBuf;

use swc_common::errors::{EmitterWriter, Handler};
use swc_common::input::SourceFileInput;
use swc_common::FileName;
use swc_common::{sync::Lrc, SourceMap};
use swc_common::{Mark, GLOBALS};
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Parser};
use swc_ecma_parser::{Capturing, EsConfig, Syntax};
use swc_ecma_preset_env::preset_env;
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::FoldWith;

use std::sync::Arc;

pub fn parse_js_2_ast(file_name: &str, code: &str) -> (Module, Arc<String>) {
    return GLOBALS.set(&Default::default(), || {
        // code = code.trim().replace("\n", "").replace("\r", "");
        let source_map = Lrc::new(SourceMap::default());

        let handler = Handler::with_emitter(
            true,
            false,
            Box::new(EmitterWriter::new(
                Box::new(std::io::stderr()),
                Some(source_map.clone()),
                false,
                false,
            )),
        );

        let fm = source_map.new_source_file(FileName::Custom(file_name.into()), code.into());

        let lexer = Lexer::new(
            Syntax::Es(EsConfig::default()),
            Default::default(),
            SourceFileInput::from(&*fm),
            None,
        );

        let capturing = Capturing::new(lexer);

        let mut parser = Parser::new_from(capturing);
        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        (
            parser
                .parse_module()
                .map_err(|e| e.into_diagnostic(&handler).emit())
                .expect("Failed to parse module."),
            source_map
                .get_source_file(&FileName::Custom(file_name.into()))
                .expect(&format!("获取源代码失败{}", &file_name))
                .src
                .clone(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_ecma_ast::*;

    #[test]
    fn test_get_js_ast() {
        let file_name = "function.js";
        let mut code = "export function add(a, b)
                { return a + b; }
            export const test = 
            (a) => { return a }";
        let module = parse_js_2_ast(file_name, &mut code);
    }

    #[test]
    fn test_get_async_js_ast() {
        let file_name = "function.js";
        let mut code = "
        export async function add(a, b) { return a + b; }
        export const test = async (a) => { return a }";
        dbg!(&code);
        let module = parse_js_2_ast(file_name, &mut code);
        dbg!(&module);
    }
}
