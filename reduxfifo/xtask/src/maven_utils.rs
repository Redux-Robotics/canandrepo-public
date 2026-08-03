use anyhow::anyhow;
use sha1::Digest;
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use zip::write::SimpleFileOptions;

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
}

#[derive(Debug)]
pub struct ReduxFIFOCrate {
    /// workspace root
    pub workspace_root: PathBuf,
    /// target directory
    pub target_dir: PathBuf,
    /// Cargo.toml manifest
    pub manifest: cargo_toml::Manifest,
    /// crate version
    pub version: semver::Version,
    /// derive the year to get the version of the wpilib toolchain to use.
    pub year: u64,
}

impl ReduxFIFOCrate {
    pub fn new(workspace_root: &Path) -> anyhow::Result<Self> {
        let workspace_root = workspace_root.to_path_buf();
        let manifest = cargo_toml::Manifest::from_path(workspace_root.join("Cargo.toml"))?;
        let version = manifest
            .workspace
            .as_ref()
            .ok_or(anyhow!("ReduxFIFO Cargo.toml isn't a workspace"))?
            .package
            .as_ref()
            .ok_or(anyhow!("ReduxFIFO Cargo.toml lacks top-level package"))?
            .version
            .clone()
            .ok_or(anyhow!(
                "ReduxFIFO Cargo.toml is not in 'year-semver' format"
            ))?;
        let year = version.major;
        let target_dir = workspace_root.join("target");

        Ok(Self {
            workspace_root,
            target_dir,
            manifest,
            version,
            year,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolchain {
    /// Base directory containing the tools
    pub base: PathBuf,
    /// String prefix, e.g. `aarch64-systemcoreYEAR-linux-gnu`
    pub prefix: String,
}

impl Toolchain {
    pub fn tool(&self, name: &str) -> PathBuf {
        self.base.join(format!("{}-{}", self.prefix, name))
    }

    pub fn locate(prefix: String, arch: &str, year: u64) -> Result<Self, ToolchainNotFound> {
        let gcc_name = format!("{prefix}-gcc");
        if let Ok(w) = which::which(&gcc_name) {
            // sometimes the systemcore toolchain is already in PATH (e.g. in buildserver containers)
            return Ok(Self {
                base: w.parent().unwrap().to_path_buf(),
                prefix,
            });
        }

        let mut search_path = Vec::new();
        let maybe_home = std::env::home_dir();
        if let Some(home) = &maybe_home {
            // We'll prefer the gradle cache, if it exists.
            search_path.push(home.join(format!(".gradle/toolchains/first/{year}/{arch}/bin")));
        }

        #[cfg(unix)]
        {
            if let Some(home) = &maybe_home {
                // All unicies have their wpilib install in the home directory.
                // For now.
                search_path.push(home.join(format!("wpilib/{year}/{arch}/bin")));
            };
            // the cross-compiler container
            search_path.push(PathBuf::from("/usr/local/bin"));
        }

        #[cfg(windows)]
        {
            // windows typically puts the toolchain in C:\Users\Public for whatever reason
            search_path.push(
                PathBuf::from(std::env::var("PUBLIC").unwrap_or("C:\\Users\\Public".into()))
                    .join(format!("wpilib\\{year}\\{arch}\\bin")),
            );
            if let Some(home) = &maybe_home {
                search_path.push(home.join(format!("wpilib\\{year}\\{arch}\\bin")));
            }
        }

        // find gcc base path
        for path in &search_path {
            let gcc = path.join(&gcc_name);
            if gcc.exists() && gcc.is_file() {
                return Ok(Self {
                    base: path.clone(),
                    prefix,
                });
            }
        }
        Err(ToolchainNotFound {
            prefix,
            searched: search_path,
        })
    }
}

#[derive(Debug)]
pub struct ToolchainNotFound {
    pub prefix: String,
    pub searched: Vec<PathBuf>,
}
impl core::fmt::Display for ToolchainNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Toolchain {} not found, is it installed?", self.prefix)?;
        writeln!(f, "Searched in:")?;
        for path in &self.searched {
            writeln!(f, "\t{}", path.display())?;
        }
        Ok(())
    }
}
impl core::error::Error for ToolchainNotFound {}

pub fn locate_systemcore_toolchain(year: u64) -> Result<Toolchain, ToolchainNotFound> {
    Toolchain::locate(
        format!("aarch64-systemcore{year}-linux-gnu"),
        "systemcore",
        year,
    )
}

pub fn locate_aarch64_toolchain(year: u64) -> Result<Toolchain, ToolchainNotFound> {
    Toolchain::locate(format!("aarch64-trixie-linux-gnu"), "arm64", year)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    LinuxSystemCore,
    WindowsX86_64,
    WindowsArm64,
    OsxUniversal,
    OsxX86_64,
    OsxArm64,
    LinuxX86_64,
    LinuxArm64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Mac,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binary {
    /// .so that goes from `libname.so -> libname.so`
    SoLib,
    /// .so that goes from `libname.so.stripped -> libname.so`
    StrippedSoLib,
    /// macos .dylib (no debug binaries)
    DyLib,
    /// windows dll
    WinDll,
    /// windows debug artifact
    WinDbg(&'static str),
    /// debug symbol that goes from `name.dll.lib` -> `name.lib`
    WinLib,
}

impl Binary {
    pub fn source_name(&self, name: &str) -> String {
        match self {
            Self::SoLib => format!("lib{name}.so"),
            Self::StrippedSoLib => format!("lib{name}.so.stripped"),
            Self::DyLib => format!("lib{name}.dylib"),
            Self::WinDll => format!("{name}.dll"),
            Self::WinDbg(s) => format!("{name}{s}"),
            Self::WinLib => format!("{name}.dll.lib"),
        }
    }

    pub fn dest_name(&self, name: &str) -> String {
        match self {
            Self::SoLib | Self::StrippedSoLib => format!("lib{name}.so"),
            Self::DyLib => format!("lib{name}.dylib"),
            Self::WinDll => format!("{name}.dll"),
            Self::WinDbg(s) => format!("{name}{s}"),
            Self::WinLib => format!("{name}.lib"),
        }
    }
}

impl OperatingSystem {
    pub const fn name(&self) -> &'static str {
        match self {
            OperatingSystem::Linux => "linux",
            OperatingSystem::Windows => "windows",
            OperatingSystem::Mac => "osx",
        }
    }
    pub const fn release_artifacts(&self) -> &'static [Binary] {
        match self {
            OperatingSystem::Linux => &[Binary::StrippedSoLib],
            OperatingSystem::Windows => &[Binary::WinDll],
            OperatingSystem::Mac => &[Binary::DyLib],
        }
    }
    pub const fn debug_artifacts(&self) -> &'static [Binary] {
        match self {
            OperatingSystem::Linux => &[Binary::SoLib],
            OperatingSystem::Windows => &[
                Binary::WinDll,
                Binary::WinLib,
                Binary::WinDbg(".pdb"),
                Binary::WinDbg(".dll.exp"),
            ],
            OperatingSystem::Mac => &[Binary::DyLib],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Architecture {
    SystemCore,
    X86_64,
    Arm64,
    OsxUniversal,
}

impl Architecture {
    pub const fn name(&self) -> &'static str {
        match self {
            Architecture::SystemCore => "systemcore",
            Architecture::X86_64 => "x86-64",
            Architecture::Arm64 => "arm64",
            Architecture::OsxUniversal => "universal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetInfo {
    /// the rust target triple
    pub triple: &'static str,
    /// the operating system
    pub os: OperatingSystem,
    /// the "architecture" that wpilib maven expects.
    /// This is similar to but not exactly the device's actual cpu arch.
    pub arch: Architecture,
}

impl Target {
    /// information about the target.
    pub const fn info(&self) -> TargetInfo {
        match self {
            Target::LinuxSystemCore => TargetInfo {
                triple: "aarch64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::SystemCore,
            },
            Target::WindowsX86_64 => TargetInfo {
                triple: "x86_64-pc-windows-msvc",
                os: OperatingSystem::Windows,
                arch: Architecture::X86_64,
            },
            Target::WindowsArm64 => TargetInfo {
                triple: "aarch64-pc-windows-msvc",
                os: OperatingSystem::Windows,
                arch: Architecture::Arm64,
            },
            Target::OsxUniversal => TargetInfo {
                triple: "universal-apple-darwin",
                os: OperatingSystem::Mac,
                arch: Architecture::OsxUniversal,
            },
            Target::OsxArm64 => TargetInfo {
                triple: "aarch64-apple-darwin",
                os: OperatingSystem::Mac,
                arch: Architecture::Arm64,
            },
            Target::OsxX86_64 => TargetInfo {
                triple: "x86_64-apple-darwin",
                os: OperatingSystem::Mac,
                arch: Architecture::X86_64,
            },
            Target::LinuxX86_64 => TargetInfo {
                triple: "x86_64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::X86_64,
            },
            Target::LinuxArm64 => TargetInfo {
                triple: "aarch64-unknown-linux-gnu",
                os: OperatingSystem::Linux,
                arch: Architecture::Arm64,
            },
        }
    }

    /// Gets the running host target.
    pub const fn host() -> Self {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let target = Self::LinuxX86_64;
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let target = Self::LinuxArm64;
        #[cfg(target_os = "macos")]
        let target = Self::OsxUniversal;
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        let target = Self::WindowsX86_64;
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        let target = Self::WindowsArm64;

        target
    }

    pub fn build(
        &self,
        crate_info: &ReduxFIFOCrate,
        cargo_flags: &Vec<String>,
    ) -> anyhow::Result<()> {
        let lib_name = crate_info.manifest.clone().lib.unwrap().name.unwrap();
        let dir = crate_info.workspace_root.as_path();
        let info = self.info();

        match self {
            Target::LinuxSystemCore => {
                cargo_build(
                    dir,
                    &info.triple,
                    Some(&locate_systemcore_toolchain(crate_info.year)?),
                    cargo_flags,
                )?;
            }
            Target::LinuxArm64 => {
                cargo_build(
                    dir,
                    &info.triple,
                    Some(&locate_aarch64_toolchain(crate_info.year)?),
                    cargo_flags,
                )?;
            }
            Target::OsxUniversal => {
                // osxuniversal needs to build twice and then lipo all the artifacts together
                cargo_build(dir, "aarch64-apple-darwin", None, cargo_flags)?;
                cargo_build(dir, "x86_64-apple-darwin", None, cargo_flags)?;

                std::fs::create_dir_all(
                    crate_info
                        .target_dir
                        .join(format!("universal-apple-darwin/release")),
                )
                .ok();
                lipo(dir, &format!("release/lib{lib_name}.dylib"))?;
            }
            _other => {
                cargo_build(dir, &info.triple, None, cargo_flags)?;
            }
        }

        if info.os == OperatingSystem::Linux {
            // we need to generate a stripped binary
            let sysroot = String::from_utf8(
                Command::new("rustc")
                    .args(["--print", "sysroot"])
                    .output()?
                    .stdout,
            )?;

            let llvm_strip = Path::new(sysroot.trim())
                .join(format!("lib/rustlib/{}/bin/llvm-strip", Target::host().info().triple));

            eprintln!("Stripping built lib{lib_name}.so with {}", llvm_strip.display());

            let mut strip = Command::new(&llvm_strip);
            let release_dir = dir.join(format!("target/{}/release", info.triple));
            strip.arg("-g");
            strip.arg("-o");
            strip.arg(release_dir.join(format!("lib{lib_name}.so.stripped")));
            strip.arg(release_dir.join(format!("lib{lib_name}.so")));
            strip.status()?;
        }

        Ok(())
    }
}

fn lipo(dir: &Path, artifact_path: &str) -> anyhow::Result<()> {
    Command::new("lipo")
        .current_dir(dir)
        .arg("-create")
        .arg("-output")
        .arg(format!("target/universal-apple-darwin/{artifact_path}"))
        .arg(format!("target/x86_64-apple-darwin/{artifact_path}"))
        .arg(format!("target/aarch64-apple-darwin/{artifact_path}"))
        .status()?;
    Ok(())
}

fn cargo_build(
    dir: &Path,
    triple: &str,
    link_toolchain: Option<&Toolchain>,
    flags: &Vec<String>,
) -> anyhow::Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cargo = Command::new(cargo);
    cargo.current_dir(dir);
    cargo.arg("build");
    cargo.arg("--release");
    cargo.arg(format!("--target={triple}"));
    cargo.args(flags);

    // prepend the toolchain to PATH and setup environment variables
    if let Some(toolchain) = link_toolchain {
        #[cfg(unix)]
        const SEP: &str = ":";
        #[cfg(windows)]
        const SEP: &str = ";";

        let mut path = std::env::var("PATH")?;
        path = format!("{}{SEP}{path}", toolchain.base.display());

        // convert from kebab to UPPER_SNAKE
        let var_target = triple.replace("-", "_").to_uppercase();
        cargo.env(
            format!("CARGO_TARGET_{var_target}_LINKER"),
            toolchain.tool("gcc"),
        );
        cargo.env(
            format!("CARGO_TARGET_{var_target}_AR"),
            toolchain.tool("ar"),
        );
        cargo.env("PATH", path);
    }
    cargo.status()?;

    Ok(())
}

struct LowerHexAdapter<T>(T);
impl<T: AsRef<[u8]>> core::fmt::Display for LowerHexAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.0.as_ref() {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

pub fn calc_hashes(file_path: &Path) -> anyhow::Result<()> {
    let data = std::fs::read(file_path)?;
    let ext = file_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    std::fs::write(
        file_path.with_extension(format!("{ext}.md5")),
        format!("{:x}", md5::compute(&data)),
    )?;
    let mut h = sha1::Sha1::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha1")),
        LowerHexAdapter(h.finalize()).to_string(),
    )?;
    let mut h = sha2::Sha256::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha256")),
        LowerHexAdapter(h.finalize()).to_string(),
    )?;
    let mut h = sha2::Sha512::new();
    h.update(&data);
    std::fs::write(
        file_path.with_extension(format!("{ext}.sha512")),
        LowerHexAdapter(h.finalize()).to_string(),
    )?;

    Ok(())
}
#[cfg(unix)]
const PATH_SEP: &str = "/";
#[cfg(windows)]
const PATH_SEP: &str = "\\";

pub fn build_maven(
    crate_info: &ReduxFIFOCrate,
    target: Target,
    group_id: &str,
    artifact_id: &str,
    cargo_flags: &Vec<String>,
) -> anyhow::Result<()> {
    eprintln!("Building target {target:?}");
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));
    let lib_name = crate_info.manifest.clone().lib.unwrap().name.unwrap();
    let target_info = target.info();

    let build_dir = crate_info
        .target_dir
        .join(target_info.triple)
        .join("release");

    let license = std::fs::read(crate_info.workspace_root.join("LICENSE.txt"))?;

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&crate_info.version.to_string());
    eprintln!("Creating maven target {maven:?}");

    std::fs::create_dir_all(&maven).ok();
    target.build(crate_info, cargo_flags)?;

    for is_debug in [false, true] {
        let zipfname = maven.join(format!(
            "{artifact_id}-{version}-{}{}{}.zip",
            target_info.os.name(),
            target_info.arch.name(),
            if is_debug { "debug" } else { "" },
        ));
        eprintln!("Building zip {zipfname:?}");
        build_maven_binary_zip(&zipfname, &build_dir, &lib_name, target, &license, is_debug)?;
    }

    Ok(())
}

fn build_maven_binary_zip(
    fname: &Path,
    build_dir: &Path,
    lib_name: &str,
    target: Target,
    license_txt: &[u8],
    debug: bool,
) -> anyhow::Result<()> {
    let target_info = target.info();
    let zipf = std::fs::File::create(fname)?;
    let mut zip = zip::ZipWriter::new(zipf);

    zip.start_file("LICENSE.txt", zip_options())?;
    zip.write_all(license_txt)?;

    // create the os/arch/linkage/ directory
    zip.add_directory(target_info.os.name(), zip_options())?;
    let base_path = format!(
        "{}/{}/shared",
        target_info.os.name(),
        target_info.arch.name(),
    );
    zip.add_directory(&base_path, zip_options())?;

    let artifacts = if debug {
        target_info.os.debug_artifacts()
    } else {
        target_info.os.release_artifacts()
    };
    // write the artifact to the zip
    for artifact_bin in artifacts {
        let artifact_name = artifact_bin.source_name(lib_name);
        let artifact_dest = artifact_bin.dest_name(lib_name);

        zip.start_file_from_path(format!("{}/{}", &base_path, &artifact_dest), zip_options())?;
        zip.write_all(&std::fs::read(build_dir.join(artifact_name))?)?;
    }
    zip.finish()?;
    calc_hashes(&fname)?;

    Ok(())
}

pub fn build_maven_zip(
    crate_info: &ReduxFIFOCrate,
    root_path: &Path,
    group_id: &str,
    artifact_id: &str,
    artifact_name: &str,
) -> anyhow::Result<()> {
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version.to_string());
    std::fs::create_dir_all(&maven).ok();
    let zipfname = &maven.join(format!("{artifact_id}-{version}-{artifact_name}.zip"));
    let zipf = std::fs::File::create(zipfname)?;
    let mut zip = zip::ZipWriter::new(zipf);
    zip.start_file("LICENSE.txt", zip_options())?;
    zip.write_all(std::fs::read(crate_info.workspace_root.join("LICENSE.txt"))?.as_slice())?;

    for entry in walkdir::WalkDir::new(root_path).into_iter() {
        let ent = entry?;
        if ent.path() == root_path {
            continue;
        }
        let Ok(relpath) = ent.path().strip_prefix(root_path) else {
            continue;
        };

        if ent.file_type().is_file() {
            zip.start_file_from_path(relpath, zip_options())?;
            zip.write_all(std::fs::read(ent.path())?.as_slice())?;
        } else if ent.file_type().is_dir() {
            zip.add_directory_from_path(ent.path(), zip_options())?;
        }
    }
    zip.finish()?;
    calc_hashes(&zipfname)?;
    Ok(())
}

pub fn build_maven_metadata(
    crate_info: &ReduxFIFOCrate,
    group_id: &str,
    artifact_id: &str,
) -> anyhow::Result<()> {
    eprintln!("Building maven-metadata.xml file");
    let version = crate_info.version.to_string();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", PATH_SEP)));

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id);
    std::fs::create_dir_all(&maven).ok();

    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let maven_metadata = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<metadata>
  <groupId>{group_id}</groupId>
  <artifactId>{artifact_id}</artifactId>
  <versioning>
    <latest>{version}</latest>
    <release>{version}</release>
    <versions>
      <version>{version}</version>
    </versions>
    <lastUpdated>{ts}</lastUpdated>
  </versioning>
</metadata>"
    );
    let maven_metadata_path = maven.join("maven-metadata.xml");
    std::fs::write(&maven_metadata_path, maven_metadata)?;
    calc_hashes(maven_metadata_path.as_path())?;
    Ok(())
}

pub fn build_maven_pom(
    crate_info: &ReduxFIFOCrate,
    group_id: &str,
    artifact_id: &str,
) -> anyhow::Result<()> {
    eprintln!("Building POM file");
    let version = crate_info.version.clone();
    let group_id_as_path = PathBuf::from(OsString::from(group_id.replace(".", "/")));

    let maven = crate_info
        .target_dir
        .join("maven")
        .join(group_id_as_path)
        .join(artifact_id)
        .join(&version.to_string());
    std::fs::create_dir_all(&maven).ok();

    let maven_pom_data = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<project xsi:schemaLocation=\"http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd\" xmlns=\"http://maven.apache.org/POM/4.0.0\"
    xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">
  <modelVersion>4.0.0</modelVersion>
  <groupId>{group_id}</groupId>
  <artifactId>{artifact_id}</artifactId>
  <version>{version}</version>
  <packaging>pom</packaging>
</project>"
    );
    let maven_pom_path = maven.join(format!("{artifact_id}-{version}.pom"));
    std::fs::write(&maven_pom_path, maven_pom_data)?;
    calc_hashes(maven_pom_path.as_path())?;
    Ok(())
}
