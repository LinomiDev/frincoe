use std::io::Read;

use proc_macro::{Diagnostic, Level};
use syn::spanned::Spanned;
use syn::{Item, LitStr, Path, TraitItem};

macro_rules! report_error (
    ( $var:ident, $msg:expr ) => {
        {
            Diagnostic::spanned($var.span().unwrap(), Level::Error, $msg).emit();
            return None;
        }
    }
);

fn read_file(fname: &str) -> Result<syn::File, Box<dyn std::error::Error>> {
    let mut handle = std::fs::File::open(fname)?;
    let mut content = String::new();
    handle.read_to_string(&mut content)?;
    Ok(syn::parse_file(&content)?)
}

pub fn read_trait(fname: LitStr, srcpath: &Path) -> Option<Vec<TraitItem>> {
    // Parse the file and locate the trait to be injected
    let mut content = match read_file(
        proc_macro::Span::call_site()
            .source_file()
            .path()
            .parent()
            .unwrap()
            .join(fname.value())
            .to_str()
            .unwrap(),
    ) {
        Ok(v) => v.items,
        Err(err) => report_error!(fname, format!("failed when parsing the trait source: {}", err)),
    };
    if srcpath.segments.is_empty() {
        report_error!(srcpath, "expected the trait name");
    }
    // Match the mod path...
    for curpath in srcpath.segments.iter().take(srcpath.segments.len() - 1) {
        let mut next: Option<Vec<Item>> = None;
        for item in content {
            match item {
                Item::Mod(item) if item.ident == curpath.ident => {
                    next = item.content.map(|x| x.1);
                    break;
                }
                _ => (),
            }
        }
        if let Some(next) = next {
            content = next;
        } else {
            report_error!(srcpath, "expected the trait name");
        }
    }
    // Find the trait in the mod...
    let name = &srcpath.segments.last().unwrap().ident;
    let mut next: Option<Vec<TraitItem>> = None;
    for item in content {
        match item {
            Item::Trait(item) if item.ident == *name => {
                next = Some(item.items);
                break;
            }
            _ => (),
        }
    }

    match next {
        Some(v) => Some(v),
        None => report_error!(srcpath, "not found the trait with the given path"),
    }
}
