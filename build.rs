use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn get_git_commit_version() {
    let result = std::process::Command::new("git")
        .args(&["describe", "--tags", "--always", "--dirty"])
        .output();

    let git_version = match result {
        Ok(output) => output.stdout,
        Err(_) => Vec::new(),
    };

    let git_version = String::from_utf8(git_version).expect("Invalid UTF-8 sequence");
    println!("cargo:rustc-env=GIT_VERSION={}", git_version.trim());
}

fn link_rename_lib() {
    /*
    rename the output file to smartdns_ui.so
    */
    let release_plugin = env::var("RELEASE_PLUGIN").is_ok();

    if release_plugin == false {
        // In debug mode, we don't rename the output file
        return;
    }

    let curr_source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir =
        env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| format!("{}/target", curr_source_dir));
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap().replace("-", "_");
    let so_path = format!("{}/{}.so", target_dir, crate_name);
    println!("cargo:rustc-link-arg=-o");
    println!("cargo:rustc-link-arg={}", so_path);
}

fn should_link_zlib() -> bool {
    matches!(
        env::var("WITH_ZLIB")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "1" | "yes" | "true" | "on"
    )
}

fn link_smartdns_lib() {
    let curr_source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let smartdns_inc_dir = env::var("SMARTDNS_INCLUDE_DIR")
        .unwrap_or_else(|_| format!("{}/vendor/include", curr_source_dir));
    let smartdns_lib_file =
        env::var("SMARTDNS_TEST_LIB").unwrap_or_else(|_| String::from("__missing__"));

    let cc = env::var("RUSTC_LINKER")
        .unwrap_or_else(|_| env::var("CC").unwrap_or_else(|_| "cc".to_string()));

    let sysroot_output = std::process::Command::new(&cc)
        .arg("--print-sysroot")
        .output();
    let mut sysroot = None;
    if let Ok(output) = sysroot_output {
        if output.status.success() {
            let path = String::from_utf8(output.stdout).unwrap();
            sysroot = Some(path.trim().to_string());
        }
    }

    let ignored_macros = IgnoreMacros(vec!["IPPORT_RESERVED".into()].into_iter().collect());

    let mut bindings_builder =
        bindgen::Builder::default().header(format!("{}/smartdns/smartdns.h", smartdns_inc_dir));
    if let Some(sysroot) = sysroot {
        bindings_builder = bindings_builder.clang_arg(format!("--sysroot={}", sysroot));
    }
    let bindings = bindings_builder
        .clang_arg(format!("-I{}", smartdns_inc_dir))
        .parse_callbacks(Box::new(ignored_macros))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("smartdns_bindings.rs"))
        .expect("Couldn't write bindings!");
    /*
    to run tests, please run the following command:
    make test-prepare
    */
    if std::path::Path::new(&smartdns_lib_file).exists() && !cfg!(feature = "build-release") {
        println!("cargo:rerun-if-changed={}", smartdns_lib_file);
        println!(
            "cargo:rustc-link-search=native={}",
            PathBuf::from(&smartdns_lib_file)
                .parent()
                .unwrap()
                .display()
        );
        println!("cargo:rustc-link-lib=static=smartdns-test");
        println!("cargo:rustc-link-lib=ssl");
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rerun-if-env-changed=WITH_ZLIB");
        if should_link_zlib() {
            println!("cargo:rustc-link-lib=z");
        }
    }
}

fn main() {
    get_git_commit_version();
    link_smartdns_lib();
    link_rename_lib();
}
