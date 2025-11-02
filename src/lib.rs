use std::sync::Arc;

use typst::{
    Library, LibraryExt, WorldExt,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use wasm_bindgen::prelude::*;

mod iface;
mod utils;

struct BasicWorld {
    fonts: Vec<Font>,
    font_book: LazyHash<FontBook>,
    library: LazyHash<Library>,
    root: FileId,
}

struct World {
    shared: Arc<BasicWorld>,
    source: String,
}

impl typst::World for World {
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

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.shared.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

#[wasm_bindgen]
pub struct Context {
    // This can arguably just be a reference in World, using Arcs as a simplification for now
    basic_world: Arc<BasicWorld>,
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
    let root = FileId::new_fake(VirtualPath::new("/root"));
    Context {
        basic_world: Arc::new(BasicWorld {
            fonts,
            font_book,
            library,
            root,
        }),
    }
}

#[wasm_bindgen]
pub fn compile(
    context: &Context,
    source: &str,
    px_per_pt: f32,
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
        shared: context.basic_world.clone(),
        source: prefix + source,
    };
    let result = typst::compile::<PagedDocument>(&world);

    let mut diagnostics: Vec<_> = result
        .warnings
        .into_iter()
        .map(|diag| iface::Diagnostic::from_source_diagnostic(&world, prefix_len, diag))
        .collect();

    let output = match result.output {
        Ok(doc) => {
            let pm = typst_render::render_merged(&doc, px_per_pt, typst::layout::Abs::zero(), None);

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
