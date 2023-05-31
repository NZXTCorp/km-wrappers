use std::{env, path::Path};

fn main() {
    if env::var_os("CARGO_FEATURE_LINKING").is_none() {
        // don't wanna link
        return;
    }

    if let Ok(env_file) = dotenvy::dotenv() {
        println!("cargo:rerun-if-changed={}", env_file.display());
    }

    let lib_km = env::var_os("KM_RS_WDK_LIB_KM_64").expect("`KM_RS_WDK_LIB_KM_64` was not set");
    let lib_kmdf =
        env::var_os("KM_RS_WDK_LIB_KMDF_64").expect("`KM_RS_WDK_LIB_KMDF_64` was not set");

    println!("cargo:rustc-link-search={}", Path::new(&lib_km).display());
    println!("cargo:rustc-link-search={}", Path::new(&lib_kmdf).display());
}
