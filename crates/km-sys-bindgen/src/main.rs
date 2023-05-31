#![deny(rust_2018_idioms)]

use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct BindgenConfig {
    enums: BindgenEnumConfig,
    allowlists: BindgenAllowlists,
}

#[derive(Deserialize)]
struct BindgenEnumConfig {
    bitfield_enums: Vec<String>,
    constified_enums: Vec<String>,
    rustified_enums: Vec<String>,
    newtype_enums: Vec<String>,
}

#[derive(Deserialize)]
struct BindgenAllowlists {
    allowed_functions: Vec<String>,
    allowed_vars: Vec<String>,
    allowed_types: Vec<String>,
}

fn main() {
    let out_file = env::args()
        .nth(1)
        .expect("USAGE: km-sys-bindgen.exe <outfile>");

    dotenvy::dotenv().ok();

    let shared_includes =
        env::var("KM_RS_WDK_INCLUDE_SHARED").expect("`KM_RS_WDK_INCLUDE_SHARED` was not set");
    let km_includes = env::var("KM_RS_WDK_INCLUDE_KM").expect("`KM_RS_WDK_INCLUDE_KM` was not set");
    let kmdf_includes =
        env::var("KM_RS_WDK_INCLUDE_WDM_KMDF").expect("`KM_RS_WDK_INCLUDE_WDM_KMDF` was not set");

    let BindgenConfig {
        allowlists:
            BindgenAllowlists {
                allowed_functions,
                allowed_types,
                allowed_vars,
            },
        enums:
            BindgenEnumConfig {
                bitfield_enums,
                constified_enums,
                rustified_enums,
                newtype_enums,
            },
    } = toml::from_str(include_str!("../bindgen.toml"))
        .expect("Could not deserialize `bindgen.toml`");

    let mut builder = bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("::libc")
        .header("bindgen.h")
        .clang_args([
            format!("-I{shared_includes}"),
            format!("-I{km_includes}"),
            format!("-I{kmdf_includes}"),
        ])
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .layout_tests(false)
        .rustfmt_bindings(true);

    for f in allowed_functions {
        builder = builder.allowlist_function(f);
    }

    for t in allowed_types {
        builder = builder.allowlist_type(t);
    }

    for v in allowed_vars {
        builder = builder.allowlist_var(v);
    }

    for e in bitfield_enums {
        builder = builder.bitfield_enum(e);
    }

    for e in constified_enums {
        builder = builder.constified_enum(e);
    }

    for e in rustified_enums {
        builder = builder.rustified_enum(e);
    }

    for e in newtype_enums {
        builder = builder.newtype_enum(e);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_file)
        .expect("Couldn't write bindings");

    println!("\n\nBindings generated successfully");
}
