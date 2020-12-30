/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};

const POSSIBLE_LANGUAGES: &[&str] = &["kotlin", "python", "swift", "gecko_js"];

fn main() -> Result<()> {
    let matches = clap::App::new("uniffi-bindgen")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .subcommand(
            clap::SubCommand::with_name("generate")
                .about("Generate foreign language bindings")
                .arg(
                    clap::Arg::with_name("language")
                        .required(true)
                        .takes_value(true)
                        .long("--language")
                        .short("-l")
                        .multiple(true)
                        .number_of_values(1)
                        .possible_values(&POSSIBLE_LANGUAGES)
                        .help("Foreign language(s) for which to build bindings"),
                )
                .arg(
                    clap::Arg::with_name("out_dir")
                        .long("--out-dir")
                        .short("-o")
                        .takes_value(true)
                        .help("Directory in which to write generated files. Default is same folder as .udl file."),
                )
                .arg(
                    clap::Arg::with_name("no_format")
                        .long("--no-format")
                        .help("Do not try to format the generated bindings"),
                )
                .arg(clap::Arg::with_name("udl_file").required(true))
                .arg(
                    clap::Arg::with_name("config")
                    .long("--config-path")
                    .takes_value(true)
                    .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("scaffolding")
                .about("Generate Rust scaffolding code")
                .arg(
                    clap::Arg::with_name("out_dir")
                        .long("--out-dir")
                        .short("-o")
                        .takes_value(true)
                        .help("Directory in which to write generated files. Default is same folder as .udl file."),
                )
                .arg(
                    clap::Arg::with_name("manifest")
                    .long("--manifest-path")
                    .takes_value(true)
                    .help("Path to crate's Cargo.toml. If not provided, Cargo.toml is assumed to be in the UDL's file parent folder.")
                )
                .arg(
                    clap::Arg::with_name("config")
                    .long("--config-path")
                    .takes_value(true)
                    .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
                )
                .arg(
                    clap::Arg::with_name("no_format")
                        .long("--no-format")
                        .help("Do not format the generated code with rustfmt (useful for maintainers)"),
                )
                .arg(clap::Arg::with_name("udl_file").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("test")
            .about("Run test scripts against foreign language bindings")
            .arg(clap::Arg::with_name("cdylib_dir").required(true).help("Path to the directory containing the cdylib the scripts will be testing against."))
            .arg(clap::Arg::with_name("udl_file").required(true))
            .arg(clap::Arg::with_name("test_scripts").required(true).multiple(true).help("Foreign language(s) test scripts to run"))
            .arg(
                clap::Arg::with_name("config")
                .long("--config-path")
                .takes_value(true)
                .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess from the UDL's file location.")
            )
        )
        .get_matches();
    match matches.subcommand() {
        ("generate", Some(m)) => uniffi_bindgen::generate_bindings(
            m.value_of_os("udl_file").unwrap(), // Required
            m.value_of_os("config"),
            m.values_of("language").unwrap().collect(), // Required
            m.value_of_os("out_dir"),
            !m.is_present("no_format"),
        )?,
        ("scaffolding", Some(m)) => uniffi_bindgen::generate_component_scaffolding(
            m.value_of_os("udl_file").unwrap(), // Required
            m.value_of_os("config"),
            m.value_of_os("out_dir"),
            m.value_of_os("manifest"),
            !m.is_present("no_format"),
        )?,
        ("test", Some(m)) => uniffi_bindgen::run_tests(
            m.value_of_os("cdylib_dir").unwrap(),           // Required
            m.value_of_os("udl_file").unwrap(),             // Required
            m.values_of("test_scripts").unwrap().collect(), // Required
            m.value_of_os("config"),
        )?,
        _ => bail!("No command specified; try `--help` for some help."),
    }
    Ok(())
}
