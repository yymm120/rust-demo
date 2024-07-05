#![allow(clippy::needless_update)]
#![allow(unused)]
#![allow(unused_imports)]

/// Use memory allocator
extern crate swc_malloc;

use std::{
    collections::HashMap,
    env, fs,
    path::Path,
    time::{Duration, Instant},
};

use anyhow::Error;
use swc_bundler::{Bundle, Bundler, Load, ModuleData, ModuleRecord};
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, Mark, SourceMap, Span, GLOBALS,
};
use swc_ecma_ast::*;
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_loader::{
    resolvers::{lru::CachingResolver, node::NodeModulesResolver},
    TargetEnv,
};
use swc_ecma_minifier::option::{
    CompressOptions, ExtraOptions, MangleOptions, MinifyOptions, TopLevelOptions,
};
use swc_ecma_parser::{parse_file_as_module, Syntax};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_visit::VisitMutWith;

fn print_bundles(cm: Lrc<SourceMap>, modules: Vec<Bundle>, minify: bool) {
    // 遍历bundles, 输出每一个bundle到文件中
    for bundled in modules {
        // 1. 生成code
        let code = {
            let mut buf = vec![];

            {
                // 构造一个JsWritter
                let wr = JsWriter::new(cm.clone(), "\n", &mut buf, None);
                // 构造一个事件emitter
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default().with_minify(true),
                    cm: cm.clone(),
                    comments: None,
                    wr: if minify {
                        Box::new(omit_trailing_semi(wr)) as Box<dyn WriteJs>
                    } else {
                        Box::new(wr) as Box<dyn WriteJs>
                    },
                };
                // 触发这个事件
                emitter.emit_module(&bundled.module).unwrap();
            }

            String::from_utf8_lossy(&buf).to_string()
        };

        #[cfg(feature = "concurrent")]
        rayon::spawn(move || drop(bundled));

        // 2. 输出code
        println!("Created output.js ({}kb)", code.len() / 1024);
        fs::write("output.js", &code).unwrap();
    }
}

fn do_test(_entry: &Path, entries: HashMap<String, FileName>, inline: bool, minify: bool) {
    // 执行测试, 在run_test2中构造cm(sourceMap),并执行匿名函数
    testing::run_test2(false, |cm, _| {

        // 运行时间
        let start = Instant::now();

        // 1. 全局global, leak防止内存泄漏
        let globals = Box::leak(Box::default());
        // 2. 构造一个Bundler打包器, 传入globals, sourceMap, Loader, CachingResolver, Config, Hook
        let mut bundler = Bundler::new(
            // var1: globals
            globals,
            // var2: sourceMap
            cm.clone(),
            // var3: Loader
            Loader { cm: cm.clone() },
            // var4: CachingResolver
            CachingResolver::new(
                4096,
                NodeModulesResolver::new(TargetEnv::Node, Default::default(), true),
            ),
            // var5: Config
            swc_bundler::Config {
                require: false,
                disable_inliner: !inline,
                external_modules: Default::default(),
                disable_fixer: minify,
                disable_hygiene: minify,
                disable_dce: false,
                module: Default::default(),
            },
            // var6: Hook
            Box::new(Hook),
        );

        // 3. 执行bundler打包器的bundle方法, 生成Vec<bundle>
        // 解析代码应该是在这里。现在先不关心如何解析代码
        let mut modules = bundler
            .bundle(entries)
            .map_err(|err| println!("{:?}", err))?;
        println!("Bundled as {} modules", modules.len());

        // 打印查看一下bundle, 输出如下
        // ==> var1: [Bundle { kind: Named { name: "main" }, id: ModuleId(0), module: Module { span: 0..0#0, body ....
        println!("==> var1: {:?}", modules);
        
        #[cfg(feature = "concurrent")]
        rayon::spawn(move || {
            drop(bundler);
        });

        {
            let dur = start.elapsed();
            println!("Bundler.bundle() took {}", to_ms(dur));
        }

        let error = false;
        // 4. 如果需要压缩, 执行压缩
        if minify {
            let start = Instant::now();

            modules = modules
                .into_iter()
                .map(|mut b| {
                    GLOBALS.set(globals, || {
                        b.module = swc_ecma_minifier::optimize(
                            b.module.into(),
                            cm.clone(),
                            None,
                            None,
                            &MinifyOptions {
                                compress: Some(CompressOptions {
                                    top_level: Some(TopLevelOptions { functions: true }),
                                    ..Default::default()
                                }),
                                mangle: Some(MangleOptions {
                                    top_level: Some(true),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            &ExtraOptions {
                                unresolved_mark: Mark::new(),
                                top_level_mark: Mark::new(),
                            },
                        )
                        .expect_module();
                        b.module.visit_mut_with(&mut fixer(None));
                        b
                    })
                })
                .collect();

            let dur = start.elapsed();
            println!("Minification took {}", to_ms(dur));
        }

        // 5, 打印bundles
        {
            let cm = cm;
            print_bundles(cm, modules, minify);
        }

        if error {
            return Err(());
        }

        Ok(())
    })
    .expect("failed to process a module");
}

fn to_ms(dur: Duration) -> String {
    format!("{}ms", dur.as_millis())
}

pub fn example_swc_bundler_entry() -> Result<(), Error> {
    // 1. 读取环境变量
    let minify = env::var("MINIFY").unwrap_or_else(|_| "0".to_string()) == "1";

    // 2. 获取入口main文件
    println!("{:?}", env::args().nth(1));
    let main_file = env::args().nth(1).unwrap();
    let mut entries = HashMap::default();
    entries.insert("main".to_string(), FileName::Real(main_file.clone().into()));

    // println!("{:?}", entries);
    println!("==> var1: {:?}\n==> var2: {:?}", entries, Path::new(&main_file));
    println!("==> var1: {:?}", minify);
    
    
    // 运行时间
    let start = Instant::now();
    // 执行测试
    do_test(Path::new(&main_file), entries, false, minify);
    let dur = start.elapsed();
    println!("Took {}", to_ms(dur));

    
    Ok(())
}

struct Hook;

impl swc_bundler::Hook for Hook {
    fn get_import_meta_props(
        &self,
        span: Span,
        module_record: &ModuleRecord,
    ) -> Result<Vec<KeyValueProp>, Error> {
        let file_name = module_record.file_name.to_string();

        Ok(vec![
            KeyValueProp {
                key: PropName::Ident(Ident::new("url".into(), span)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span,
                    raw: None,
                    value: file_name.into(),
                }))),
            },
            KeyValueProp {
                key: PropName::Ident(Ident::new("main".into(), span)),
                value: Box::new(if module_record.is_entry {
                    Expr::Member(MemberExpr {
                        span,
                        obj: Box::new(Expr::MetaProp(MetaPropExpr {
                            span,
                            kind: MetaPropKind::ImportMeta,
                        })),
                        prop: MemberProp::Ident(Ident::new("main".into(), span)),
                    })
                } else {
                    Expr::Lit(Lit::Bool(Bool { span, value: false }))
                }),
            },
        ])
    }
}

pub struct Loader {
    pub cm: Lrc<SourceMap>,
}

impl Load for Loader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let fm = match f {
            FileName::Real(path) => self.cm.load_file(path)?,
            _ => unreachable!(),
        };

        let module = parse_file_as_module(
            &fm,
            Syntax::Es(Default::default()),
            EsVersion::Es2020,
            None,
            &mut vec![],
        )
        .unwrap_or_else(|err| {
            let handler =
                Handler::with_tty_emitter(ColorConfig::Always, false, false, Some(self.cm.clone()));
            err.into_diagnostic(&handler).emit();
            panic!("failed to parse")
        });

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}