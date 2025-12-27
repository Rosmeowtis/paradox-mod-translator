use walkdir::WalkDir;

fn main() {
    std::fs::create_dir_all("build").unwrap();
    std::fs::create_dir_all("build/localisation").unwrap();

    // 如果可执行文件还没有编译，先编译它
    let exe = copy_executable();
    match exe {
        Ok(_) => {} // 成功复制可执行文件
        Err(_e) => {
            return; // 直接退出 build 脚本，等待下一次构建
        }
    }

    copy_data_dir();
    copy_doc_dir();
    make_version_file();

    package_build_directory();
}

fn copy_doc_dir() {
    // 4. 复制 README.md 和 docs/ 到 build/ 目录
    std::fs::copy("README.md", "build/README.md").unwrap();
    std::fs::create_dir_all("build/docs").unwrap();
    for entry in WalkDir::new("docs") {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative_path = path.strip_prefix("docs").unwrap();
        let dest_path = std::path::Path::new("build/docs").join(relative_path);
        if path.is_dir() {
            std::fs::create_dir_all(&dest_path).unwrap();
        } else {
            std::fs::copy(path, &dest_path).unwrap();
        }
    }
}

fn copy_data_dir() {
    // 2. 复制 data 文件到 build/data 目录
    std::fs::create_dir_all("build/data").unwrap();
    for entry in WalkDir::new("data") {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative_path = path.strip_prefix("data").unwrap();
        let dest_path = std::path::Path::new("build/data").join(relative_path);
        if path.is_dir() {
            std::fs::create_dir_all(&dest_path).unwrap();
        } else {
            std::fs::copy(path, &dest_path).unwrap();
        }
    }
    std::fs::copy("task.template.toml", "build/task.template.toml").unwrap();
    std::fs::write("build/.env", "# OPENAI_API_KEY=<your api key here>\n").unwrap();
}

/// 复制 pmt 可执行文件到 build/ 目录
fn copy_executable() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    {
        #[cfg(target_os = "windows")]
        std::fs::copy("target/debug/pmt.exe", "build/pmt.exe")?;
        #[cfg(not(target_os = "windows"))]
        std::fs::copy("target/debug/pmt", "build/pmt")?;
    }
    #[cfg(not(debug_assertions))]
    {
        #[cfg(target_os = "windows")]
        std::fs::copy("target/release/pmt.exe", "build/pmt.exe")?;
        #[cfg(not(target_os = "windows"))]
        std::fs::copy("target/release/pmt", "build/pmt")?;
    }
    Ok(())
}

/// 生成 version.txt 文件
fn make_version_file() {
    let version_info = format!(
        r#"{}
Version: {}
HomePage: {}
"#,
        if cfg!(debug_assertions) {
            "Debug Mode"
        } else {
            "Release Mode"
        },
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY"),
    );
    std::fs::write("build/version.txt", version_info).unwrap();
}

/// 将 build/ 目录中的内容打包成 zip 文件并存入 dist/ 目录
fn package_build_directory() {
    std::fs::create_dir_all("dist").unwrap();
    let zip_file_path = format!(
        "dist/pmt-{}-{}.zip",
        env!("CARGO_PKG_VERSION"),
        std::env::var("TARGET").unwrap_or("unknown".to_string())
    );
    let file = std::fs::File::create(&zip_file_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for entry in WalkDir::new("build") {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative_path = path.strip_prefix("build").unwrap();
        if path.is_file() {
            zip.start_file(relative_path.to_string_lossy(), options)
                .unwrap();
            let mut f = std::fs::File::open(path).unwrap();
            std::io::copy(&mut f, &mut zip).unwrap();
        } else if relative_path.as_os_str().len() > 0 {
            zip.add_directory(relative_path.to_string_lossy(), options)
                .unwrap();
        }
    }
    zip.finish().unwrap();
}
