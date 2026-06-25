use typst::{
    Library, LibraryExt,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::{LazyHash, Scalar},
};
use typst_render::RenderOptions;
use wasm_bindgen::prelude::*;

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
pub fn load_font(
    context: &mut Context,
    data: Vec<u8>,
) -> Option<String> {
    let Some(font) = Font::new(Bytes::new(data), 0 as u32) else { return None; };
    let name = font.info().family.clone();
    context.basic_world.font_book.push(font.info().clone());
    context.basic_world.fonts.push(font);
    Some(name)
}

#[wasm_bindgen]
pub fn compile(
    context: &Context,
    source: &str,
    pixel_per_pt: f64,
    autosize: bool,
    transparent: bool,
) -> iface::CompileResult {
    let mut prefix = "".to_string();
    if autosize {
        prefix.push_str("#set page(width: auto, height: auto, margin: 0.5cm)\n");
    }
    if transparent {
        prefix.push_str("#set page(fill: none)\n");
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

    let output = match result.output {
        Ok(doc) => {
            let opts = RenderOptions { pixel_per_pt: Scalar::from(pixel_per_pt), render_bleed: false };
            let pm = typst_render::render_merged(&doc, &opts, typst::layout::Abs::zero(), None);

            let png = pm.encode_png().expect("Encoding failed");

            Some(png)
        }
        Err(err) => {
            diagnostics.extend(
                err.into_iter().map(|diag| {
                    iface::Diagnostic::from_source_diagnostic(&world, prefix_len, diag)
                }),
            );
            None
        }
    };

    iface::CompileResult {
        output,
        diagnostics,
    }
}
