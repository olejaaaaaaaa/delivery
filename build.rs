//! Builder script for compiling shaders to SPIR-V format using glslc and dxc
//! command line tools
use std::fs::{create_dir_all, read_dir};
use std::path::Path;
use std::process::Command;

enum ShaderFormat<'a> {
    Glsl { name: &'a str, ext: &'a str },
    Hlsl { name: &'a str, profile: &'a str },
}

fn get_shader_format<'a>(path: &'a Path) -> Option<ShaderFormat<'a>> {
    let ext = path.extension()?.to_str()?;
    let stem = path.file_stem()?.to_str()?;
    let name = path.file_prefix()?.to_str()?;

    match ext {
        "frag" | "vert" => Some(ShaderFormat::Glsl { name, ext }),
        "hlsl" => {
            let profile = if stem.ends_with("_vs") {
                "vs_6_6"
            } else if stem.ends_with("_ps") {
                "ps_6_6"
            } else {
                "cs_6_6"
            };
            Some(ShaderFormat::Hlsl { name, profile })
        },
        _ => None,
    }
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);

    let shaders_dir = project_root.join("./assets/shaders");
    let spv_output_dir = project_root.join("./assets/shaders/spv");

    let _ = create_dir_all(&spv_output_dir);

    if !shaders_dir.exists() {
        println!(
            "cargo:warning=shaders directory not found: {}",
            shaders_dir.display()
        );
        return;
    }

    if !spv_output_dir.exists() {
        println!(
            "cargo:warning=shaders output directory not found: {}",
            spv_output_dir.display()
        );
        return;
    }

    let dirs = read_dir(&shaders_dir).unwrap();

    for entry in dirs {
        let entry = entry.unwrap();
        let file_path = entry.path();

        if file_path.is_dir() {
            continue;
        }

        if let Some(format) = get_shader_format(&file_path) {
            match format {
                ShaderFormat::Glsl { name, ext } => {
                    let out_name = format!("{}_{}.spv", name, ext);
                    let output_path = spv_output_dir.join(&out_name);

                    let program = Command::new("glslc")
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .arg("-O0")
                        .arg(&file_path)
                        .arg("-o")
                        .arg(&output_path)
                        .spawn();

                    if let Ok(child) = program {
                        let output = child.wait_with_output().unwrap();
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            for line in stderr.lines() {
                                println!("cargo:warning={}", line);
                            }
                        }
                    } else {
                        println!("cargo:warning=glslc program not found");
                        return;
                    }
                },
                ShaderFormat::Hlsl { name, profile } => {
                    let out_name = format!("{}.spv", name);
                    let output_path = spv_output_dir.join(&out_name);

                    let program = Command::new("dxc")
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .arg(&file_path)
                        .arg("-spirv")
                        .arg("-T")
                        .arg(profile)
                        .arg("-E")
                        .arg("main")
                        .arg("-fspv-target-env=vulkan1.0")
                        .arg("-fvk-use-dx-layout")
                        .arg("-WX")
                        .arg("-Ges")
                        .arg("-HV")
                        .arg("2021")
                        .arg("-Fo")
                        .arg(&output_path)
                        .spawn();

                    if let Ok(child) = program {
                        let output = child.wait_with_output().unwrap();
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            for line in stderr.lines() {
                                println!("cargo:warning={}", line);
                            }
                        }
                    } else {
                        println!("cargo:warning=dxc program not found");
                        return;
                    }
                },
            }
        } else {
            println!("cargo:warning=invalid shader: {:?}", file_path);
        }
    }
}