use std::{env, path::Path};

/// Adds the necessary linker arguments to link to the WDK libraries, optionally loading the closest
/// `.env` file through [`dotenvy::dotenv()`]. See `.env.sample` for an example.
pub fn link_env(load_env_file: bool) {
    if load_env_file {
        if let Ok(env_file) = dotenvy::dotenv() {
            println!("cargo:rerun-if-changed={}", env_file.display());
        }
    }

    let lib_km = env::var_os("KM_RS_WDK_LIB_KM_64").expect("`KM_RS_WDK_LIB_KM_64` was not set");
    let lib_kmdf =
        env::var_os("KM_RS_WDK_LIB_KMDF_64").expect("`KM_RS_WDK_LIB_KMDF_64` was not set");

    println!("cargo:rustc-link-search={}", Path::new(&lib_km).display());
    println!("cargo:rustc-link-search={}", Path::new(&lib_kmdf).display());
}
