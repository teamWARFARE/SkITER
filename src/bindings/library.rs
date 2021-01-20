use std::fs::File;

use anyhow::anyhow;

pub struct Library {
    pub path: String,
}

impl Library {
    pub fn bundled() -> Result<Library, &'static str> {
        let path = extract_sciter().or(Err("Couldn't extract sciter"))?;

        Ok(Library { path })
    }

    pub fn path(path: &str) -> Library {
        Library {
            path: path.to_owned(),
        }
    }
}

fn extract_sciter() -> anyhow::Result<String> {
    for file in std::fs::read_dir(std::env::temp_dir())? {
        if let Ok(entry) = file {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap_or_default()
                    .to_os_string()
                    .into_string()
                    .unwrap_or_default()
                    .starts_with("sciter")
            {
                std::fs::remove_dir_all(path).ok();
            }
        }
    }

    #[cfg(target_os = "linux")]
    let sciter = include_bytes!("../../resources/libsciter.so.zs");

    #[cfg(target_os = "windows")]
    let sciter = include_bytes!("../../resources/sciter.dll.zs");

    let tmp_dir = tempfile::Builder::new().prefix("sciter").tempdir()?;
    let tmp_file = tmp_dir.path().join("sciter.dll");

    zstd::stream::copy_decode(&sciter[..], File::create(&tmp_file)?)?;

    let path = &tmp_file.into_os_string().into_string().or(Err(anyhow!(
        "Couldn't convert temp file into appropriate path"
    )))?;

    tmp_dir.into_path();

    Ok(path.to_owned())
}
