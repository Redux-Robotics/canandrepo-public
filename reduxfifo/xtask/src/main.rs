use std::path::{Path, PathBuf};

use clap::Parser as _;
use maven_utils::{BuildConfig, Target, build_maven_zip, locate_systemcore_toolchain};

use crate::maven_utils::ReduxFIFOCrate;

pub mod maven_utils;

const GROUP_ID: &str = "com.reduxrobotics.frc";
const ARTIFACT_ID: &str = "ReduxLib-fifo";

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(num_args = 1..)]
    targets: Vec<Compileable>,
    #[arg(long = "static", help = "Compile static instead of shared binaries")]
    static_build: bool,
    #[arg(long = "debug", help = "Compile with debug symbols")]
    debug_build: bool,
    #[arg(
        long = "all",
        help = "Compile all permutations of shared/static and release/debug binaries"
    )]
    all_build: bool,
    #[arg(
        long = "workspace-root",
        help = "ReduxFIFO workspace root",
        default_value = "."
    )]
    workspace_root: PathBuf,
    #[arg(
        last = true,
        num_args = 1..,
        help = "args to pass through to Cargo"
    )]
    cargo_flags: Vec<String>,
}

impl Cli {
    fn build_configs(&self) -> Vec<BuildConfig> {
        if self.all_build {
            return vec![
                BuildConfig::Shared,
                BuildConfig::Static,
                BuildConfig::SharedDebug,
                BuildConfig::StaticDebug,
            ];
        }
        vec![match (self.static_build, self.debug_build) {
            (false, false) => BuildConfig::Shared,
            (false, true) => BuildConfig::SharedDebug,
            (true, false) => BuildConfig::Static,
            (true, true) => BuildConfig::StaticDebug,
        }]
    }
}

#[derive(clap::ValueEnum, Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Compileable {
    #[value(name = "linuxsystemcore")]
    LinuxSystemCore,
    #[value(name = "linuxx86-64")]
    LinuxX86_64,
    #[value(name = "linuxarm64")]
    LinuxArm64,
    #[value(name = "windowsx86-64")]
    WindowsX86_64,
    #[value(name = "windowsarm64")]
    WindowsArm64,
    #[value(name = "osxuniversal")]
    OsxUniversal,
    #[value(name = "headers")]
    Headers,
    #[value(name = "desktop")]
    Desktop,
    #[default]
    #[value(name = "auto")]
    Auto,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    let ci = ReduxFIFOCrate::new(&cli.workspace_root)?;
    let build_configs = cli.build_configs();
    let cargo_flags = cli.cargo_flags;
    for target in cli.targets {
        match target {
            Compileable::LinuxX86_64 => {
                build_maven(&ci, Target::LinuxX86_64, &build_configs, &cargo_flags)?
            }
            Compileable::LinuxArm64 => {
                build_maven(&ci, Target::LinuxArm64, &build_configs, &cargo_flags)?
            }
            Compileable::LinuxSystemCore => {
                build_maven(&ci, Target::LinuxSystemCore, &build_configs, &cargo_flags)?
            }
            Compileable::WindowsX86_64 => {
                build_maven(&ci, Target::WindowsX86_64, &build_configs, &cargo_flags)?
            }
            Compileable::WindowsArm64 => {
                build_maven(&ci, Target::WindowsArm64, &build_configs, &cargo_flags)?
            }
            Compileable::OsxUniversal => {
                build_maven(&ci, Target::OsxUniversal, &build_configs, &cargo_flags)?
            }
            Compileable::Headers => {
                build_maven_zip(&ci, Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
            }
            Compileable::Desktop => {
                build_maven_desktop(&ci, &build_configs, &cargo_flags)?;
            }
            Compileable::Auto => {
                // always build headers
                build_maven_zip(&ci, Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
                // always build systemcore if possible
                if locate_systemcore_toolchain().is_some() {
                    build_maven(&ci, Target::LinuxSystemCore, &build_configs, &cargo_flags)?;
                }

                // build platform-dependent targets
                build_maven_desktop(&ci, &build_configs, &cargo_flags)?;

                // build extra targets if applicable
                #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
                {
                    //if Path::new("/usr/local/aarch64-linux-gnu").exists() {
                    build_maven(&ci, Target::LinuxArm64, &build_configs, &cargo_flags)?;
                }

                #[cfg(all(target_os = "windows", not(target_arch = "aarch64")))]
                build_maven(&ci, Target::WindowsArm64, &build_configs, &cargo_flags)?
            }
        }
    }
    Ok(())
}

fn build_maven_desktop(
    crate_info: &ReduxFIFOCrate,
    build_configs: &[BuildConfig],
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let target = Target::LinuxX86_64;
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let target = Target::LinuxArm64;
    #[cfg(target_os = "macos")]
    let target = Target::OsxUniversal;
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let target = Target::WindowsX86_64;
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let target = Target::WindowsArm64;

    build_maven(&crate_info, target, &build_configs, &cargo_flags)
}

fn build_maven(
    crate_info: &ReduxFIFOCrate,
    target: Target,
    build_configs: &[BuildConfig],
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    maven_utils::build_maven(
        crate_info,
        target,
        GROUP_ID,
        ARTIFACT_ID,
        build_configs,
        cargo_flags,
    )?;
    maven_utils::build_maven_pom(crate_info, GROUP_ID, ARTIFACT_ID)?;
    maven_utils::build_maven_metadata(crate_info, GROUP_ID, ARTIFACT_ID)?;
    Ok(())
}
