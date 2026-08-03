use std::path::{Path, PathBuf};

use clap::Parser as _;
use maven_utils::{Target, build_maven_zip, locate_systemcore_toolchain};

use crate::maven_utils::ReduxFIFOCrate;
#[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
use crate::maven_utils::locate_aarch64_toolchain;

pub mod maven_utils;

const GROUP_ID: &str = "com.reduxrobotics.frc";
const ARTIFACT_ID: &str = "ReduxLib-fifo";

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(num_args = 1..)]
    targets: Vec<Compileable>,
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
    let cargo_flags = cli.cargo_flags;
    for target in cli.targets {
        match target {
            Compileable::LinuxX86_64 => build_maven(&ci, Target::LinuxX86_64, &cargo_flags)?,
            Compileable::LinuxArm64 => build_maven(&ci, Target::LinuxArm64, &cargo_flags)?,
            Compileable::LinuxSystemCore => {
                build_maven(&ci, Target::LinuxSystemCore, &cargo_flags)?
            }
            Compileable::WindowsX86_64 => build_maven(&ci, Target::WindowsX86_64, &cargo_flags)?,
            Compileable::WindowsArm64 => build_maven(&ci, Target::WindowsArm64, &cargo_flags)?,
            Compileable::OsxUniversal => build_maven(&ci, Target::OsxUniversal, &cargo_flags)?,
            Compileable::Headers => {
                build_maven_zip(&ci, Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
            }
            Compileable::Desktop => {
                build_maven_desktop(&ci, &cargo_flags)?;
            }
            Compileable::Auto => {
                // always build headers
                build_maven_zip(&ci, Path::new("include"), GROUP_ID, ARTIFACT_ID, "headers")?;
                // always build systemcore if possible
                if locate_systemcore_toolchain(ci.year).is_ok() {
                    build_maven(&ci, Target::LinuxSystemCore, &cargo_flags)?;
                }

                // build platform-dependent targets
                build_maven_desktop(&ci, &cargo_flags)?;

                // build extra targets if applicable
                #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
                if locate_aarch64_toolchain(ci.year).is_ok() {
                    build_maven(&ci, Target::LinuxArm64, &cargo_flags)?;
                }

                #[cfg(all(target_os = "windows", not(target_arch = "aarch64")))]
                build_maven(&ci, Target::WindowsArm64, &cargo_flags)?
            }
        }
    }
    Ok(())
}

fn build_maven_desktop(
    crate_info: &ReduxFIFOCrate,
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    build_maven(&crate_info, Target::host(), &cargo_flags)
}

fn build_maven(
    crate_info: &ReduxFIFOCrate,
    target: Target,
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    maven_utils::build_maven(crate_info, target, GROUP_ID, ARTIFACT_ID, cargo_flags)?;
    maven_utils::build_maven_pom(crate_info, GROUP_ID, ARTIFACT_ID)?;
    maven_utils::build_maven_metadata(crate_info, GROUP_ID, ARTIFACT_ID)?;
    Ok(())
}
