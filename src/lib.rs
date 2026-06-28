use typst::{
    Library, LibraryExt, diag::{FileError, FileResult}, foundations::{Bytes, Datetime}, syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot}, text::{Font, FontBook}, utils::{LazyHash, Scalar},
};
use typst_render::RenderOptions;
use typst_svg::SvgOptions;
use wasm_bindgen::prelude::*;

use crate::iface::{Diagnostic, PngResult, SvgResult};

mod iface;
mod utils;

struct BasicWorld {
    fonts: Vec<Font>,
    font_book: LazyHash<FontBook>,
    library: LazyHash<Library>,
    root: FileId,
}

struct World<'a> {
    shared: &'a BasicWorld,
    source: String,
}

impl<'a> typst::World for World<'a> {
    fn library(&self) -> &LazyHash<Library> {
        &self.shared.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.shared.font_book
    }

    fn main(&self) -> FileId {
        self.shared.root
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.shared.root {
            Ok(Source::new(id, self.source.clone()))
        } else {
            Err(FileError::AccessDenied)
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.shared.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}

#[wasm_bindgen]
pub struct Context {
    basic_world: Box<BasicWorld>,
}

#[wasm_bindgen]
pub fn setup() -> Context {
    static FONT_DATA: [&[u8]; 2] = [
        include_bytes!("../NewCM10-Regular.otf"),
        include_bytes!("../NewCMMath-Regular.otf"),
    ];
    log!("parsing fonts");
    let fonts: Vec<Font> = FONT_DATA
        .iter()
        .map(|data| Font::new(Bytes::new(data), 0 as u32))
        .collect::<Option<_>>()
        .expect("Failed to parse fonts");
    log!("finished parsing fonts");
    let font_book = LazyHash::new(FontBook::from_fonts(fonts.iter()));
    let library = LazyHash::new(Library::builder().build());
    let root = FileId::unique(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("/root").unwrap(),
    ));
    Context {
        basic_world: Box::new(BasicWorld {
            fonts,
            font_book,
            library,
            root,
        }),
    }
}

#[wasm_bindgen]
pub fn load_font(context: &mut Context, data: Vec<u8>) -> Option<String> {
    let Some(font) = Font::new(Bytes::new(data), 0 as u32) else {
        return None;
    };
    let name = font.info().family.clone();
    context.basic_world.font_book.push(font.info().clone());
    context.basic_world.fonts.push(font);
    Some(name)
}

fn compile_common<T: typst::foundations::Output>(
    context: &Context,
    source: &str,
    autosize: bool,
    transparent: bool,
) -> (Option<T>, Vec<Diagnostic>) {
    let mut prefix = "".to_string();
    prefix.push_str("#set text(font: \"New Computer Modern\")\n");
    if transparent {
        prefix.push_str("#set page(fill: none)\n");
    }
    if autosize {
        prefix.push_str("#set page(width: auto, height: auto, margin: 0.5cm)\n");
        prefix.push_str("#show: body => context { let w = measure(body).width; if w >= 15cm { box(width: 15cm, body) } else { body } }\n");
    }
    let prefix_len = prefix.len();
    let world = World {
        shared: &context.basic_world,
        source: prefix + source,
    };
    let result = typst::compile(&world);

    let mut diagnostics: Vec<_> = result
        .warnings
        .into_iter()
        .map(|diag| iface::Diagnostic::from_source_diagnostic(&world, prefix_len, diag))
        .collect();

    let output =
        match result.output {
            Ok(doc) => Some(doc),
            Err(err) => {
                diagnostics.extend(err.into_iter().map(|diag| {
                    iface::Diagnostic::from_source_diagnostic(&world, prefix_len, diag)
                }));
                None
            }
        };

    (output, diagnostics)
}

#[wasm_bindgen]
pub fn compile_png(context: &Context, source: &str, autosize: bool, transparent: bool, px_per_pt: f64) -> PngResult {
    let (output, diagnostics) = compile_common(context, source, autosize, transparent);

    let output = output.map(|doc| {
        let opts = RenderOptions { pixel_per_pt: Scalar::new(px_per_pt), render_bleed: false };
        let gap = typst::layout::Abs::cm(0.5);
        let pm = typst_render::render_merged(&doc, &opts, gap, None);
        pm.encode_png().unwrap()
    });

    PngResult { output, diagnostics }
}

#[wasm_bindgen]
pub fn compile_svg(context: &Context, source: &str, autosize: bool, transparent: bool) -> SvgResult {
    let (output, diagnostics) = compile_common(context, source, autosize, transparent);

    let output = output.map(|doc| {
        let opts = SvgOptions { render_bleed: false, pretty: false };
        let gap = typst::layout::Abs::cm(0.5);
        typst_svg::svg_merged(&doc, &opts, gap)
    });

    SvgResult { output, diagnostics }
}
