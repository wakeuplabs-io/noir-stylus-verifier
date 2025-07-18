use reqwest;
use std::fs;
use std::io::Cursor;
use std::path::Path;

pub async fn download_zipped_asset<T, U>(
    release_repo: T,
    release_tag: T,
    asset: T,
    dst_path: U,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsRef<str>,
    U: AsRef<Path>,
{
    let response = reqwest::get(&format!(
        "https://github.com/{}/releases/download/{}/{}.zip",
        release_repo.as_ref(),
        release_tag.as_ref(),
        asset.as_ref()
    ))
    .await?;
    let bytes = response.bytes().await?;

    // write bytes to file
    use std::fs::File;
    let mut file = File::create(dst_path.as_ref().join(asset.as_ref()))?;
    std::io::copy(&mut Cursor::new(bytes.clone()), &mut file)?;

    if !dst_path.as_ref().exists() {
        fs::create_dir_all(dst_path.as_ref())?;
    }

    let target = Path::new(dst_path.as_ref());
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    archive.extract(target)?;

    Ok(())
}
