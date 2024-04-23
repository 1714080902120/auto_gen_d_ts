// 这里就先只是简单的收集即可

use swc_common::comments::SingleThreadedComments;
use swc_common::source_map::Pos;
use swc_common::{BytePos, Mark, GLOBALS};
use swc_ecma_ast::{ExportDecl, Module};
use swc_ecma_preset_env::preset_env;
use swc_ecma_transforms_base::helpers::HELPERS;
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::{FoldWith, Visit, VisitWith};
pub fn visit_ast(mut module: Module, source_code: &str) -> (Module, Vec<Vec<String>>) {
    return GLOBALS.set(&Default::default(), || {
        return HELPERS.set(&Default::default(), || {
            let mut resolver = resolver(Default::default(), Default::default(), Default::default());

            let resolved_module = module.fold_with(&mut resolver);
            let mut my_visitor = ExportFnCollector {
                collected_codes: vec![],
                source_code,
            };
            resolved_module.visit_with(&mut my_visitor);

            let mut transformer = preset_env(
                Mark::fresh(Mark::root()),
                Some(SingleThreadedComments::default()),
                Default::default(),
                Default::default(),
                &mut Default::default(),
            );

            let transformed_module = resolved_module.fold_with(&mut transformer);

            (transformed_module, my_visitor.collected_codes)
        });
    });
}

use swc_ecma_ast::{Decl, Expr};

use crate::constants::FUNCTION_CHUNK_MAX_SIZE;

#[derive(Default, Debug)]
struct ExportFnCollector<'a> {
    collected_codes: Vec<Vec<String>>,
    source_code: &'a str,
}

// 目前只处理两种情况
// 1. export function xx () {}
// 2. export const xx = () => {}
impl<'a> ExportFnCollector<'a> {
    pub fn match_export_function_or_arrow_expr(&mut self, n: &ExportDecl) {
        match &n.decl {
            Decl::Fn(fn_decl) => {
                let span = &n.span;
                let start = span.lo();
                let end = span.hi();
                self.get_source_code(start, end, &fn_decl.ident.sym);
            }
            Decl::Var(var_decl) => {
                if !var_decl.decls.is_empty() {
                    let fir_node = &var_decl.decls[0];
                    match &fir_node.init {
                        Some(expr) => match **expr {
                            Expr::Arrow(_) => {
                                let span = &n.span;
                                let start = span.lo();
                                let end = span.hi();
                                self.get_source_code(start, end, &fir_node.name.as_ident().expect("获取箭头函数名字失败").id.sym);
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    pub fn get_source_code(&mut self, start: BytePos, end: BytePos, fn_name: &str) {
        let code = self.source_code[start.to_usize() - 1..end.to_usize() - 1].to_string();
        // 由于每次请求的token个数有上限限制，所以将整个文件内容进行切块处理
        let code_len = code.len();

        // 但是对于当个函数超过限制那就无法处理了
        if code_len > FUNCTION_CHUNK_MAX_SIZE {
            println!("函数“{}”太长，忽略", fn_name);
        }
        if self.collected_codes.is_empty() {
            self.collected_codes.push(vec![code]);
            return;
        }
        let mut last_v = self.collected_codes.pop().unwrap();

        let last_size = last_v.iter().fold(0, |pre, cur| pre + cur.len());

        if last_size + code_len > FUNCTION_CHUNK_MAX_SIZE {
            self.collected_codes.push(vec![code]);
        } else {
            last_v.push(code);
            self.collected_codes.push(last_v);
        }
    }
}

impl<'a> Visit for ExportFnCollector<'a> {
    fn visit_export_decl(&mut self, n: &ExportDecl) {
        self.match_export_function_or_arrow_expr(n);
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse::parse_js_2_ast;
    use super::*;
    #[test]
    fn test_transform() {
        let file_name = "function.js";
        let mut code = "export function add(a, b)
            { return a + b; }
        export const test = 
        (a) => { return a }";
        let mut parse_res = parse_js_2_ast(file_name, &mut code);

        let module = visit_ast(parse_res.0, &parse_res.1);

        dbg!(module);
    }

    #[test]
    fn test_transform_async_js_ast() {
        let file_name = "function.js";
        let mut code = "
        export async function add(a, b) { return a + b; }
        export const test = async (a) => { return a }";
        dbg!(&code);
        let parse_res = parse_js_2_ast(file_name, &mut code);
        let module = visit_ast(parse_res.0, &parse_res.1);

        // dbg!(module);
    }
}
