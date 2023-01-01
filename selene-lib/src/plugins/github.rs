use std::path::{Path, PathBuf};

use eyre::Context;
use sha2::Digest;

use crate::plugins::lockfile::{Lockfile, PluginLock};

// PLUGIN TODO: User specified branches
fn zip_url(author: &str, repository: &str) -> String {
    format!("https://api.github.com/repos/{author}/{repository}/zipball")
}

fn zip_url_with_commit(author: &str, repository: &str, commit: &str) -> String {
    format!("{}/{commit}", zip_url(author, repository))
}

fn valid_sha(sha: &str) -> bool {
    // 40 isn't possible now, but I wouldn't be shocked :P
    sha.len() == 7 || sha.len() == 40
}

fn hex(data: &[u8]) -> String {
    let mut output = String::with_capacity(data.len() * 2);

    for byte in data {
        output.push_str(&format!("{:02x}", byte));
    }

    output
}

fn all_files_in_directory(path: &Path) -> eyre::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(all_files_in_directory(&path)?);
        } else {
            files.push(path);
        }
    }

    Ok(files)
}

fn hash_directory(path: &Path) -> eyre::Result<String> {
    let mut hasher = sha2::Sha512::new();

    let mut files = all_files_in_directory(path)?;
    files.sort();

    for file in files {
        hasher.update(file.strip_prefix(path)?.to_str().unwrap());
        hasher.update(b"===");
        hasher.update(&std::fs::read(file)?);
    }

    Ok(hex(hasher.finalize().as_slice()))
}

fn extract_filename(content_disposition: &str) -> Option<String> {
    let mut components = content_disposition.split(';');
    if components.next()? != "attachment" {
        return None;
    }

    for component in components {
        let mut component = component.split('=');
        if component.next()?.trim() != "filename" {
            continue;
        }

        let filename = component.next()?;
        if filename.starts_with('"') && filename.ends_with('"') {
            return Some(filename[1..filename.len() - 1].to_string());
        } else {
            return Some(filename.to_string());
        }
    }

    None
}

fn extract_commit(filename: &str) -> Option<String> {
    let mut components = filename.rsplitn(2, '-');
    Some(components.next()?.to_owned())
}

fn downloaded_plugin_dir() -> eyre::Result<PathBuf> {
    Ok(dirs::cache_dir()
        .ok_or_else(|| eyre::eyre!("your platform is not supported"))?
        .join("selene")
        .join("plugins"))
}

fn strip_directory(path: &Path) -> Option<&Path> {
    let mut components = path.components();
    components.next()?;
    Some(components.as_path())
}

// PLUGIN TODO: The plugin cache file that stores your agreed to settings
// should also contain the temp directories
// If this becomes async, lockfile can become a RwLock.
// PLUGIN TODO: Failing this (like not having internet) should not kill selene
#[tracing::instrument]
pub fn resolve_github_source(source: &Path, lockfile: &mut Lockfile) -> eyre::Result<PathBuf> {
    let source_url = source.to_string_lossy().into_owned();
    let components = source_url.split('/').collect::<Vec<_>>();
    if components.len() != 3 {
        return Err(eyre::eyre!(
            "invalid github source `{}`, must be github.com/author/repository",
            source.display()
        ));
    }

    assert_eq!(components[0], "github.com");

    let author = components[1];
    let repository = components[2];

    let plugin_lock = lockfile.get(&source_url);

    let url;

    if let Some(plugin_lock) = plugin_lock {
        let extract_path =
            downloaded_plugin_dir()?.join(format!("{author}-{repository}-{}", plugin_lock.commit));

        if extract_path.exists() {
            // PLUGIN TODO: Validate hash
            return Ok(extract_path);
        }

        url = zip_url_with_commit(author, repository, &plugin_lock.commit);
    } else {
        url = zip_url(author, repository);
    }

    tracing::debug!("making request to {url}");

    let response = match ureq::get(&url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(..)) => eyre::bail!(
            "couldn't find github repository `{source_url}`. make sure the repository exists and that you have internet access."
        ),
        Err(error) => {
            return Err(eyre::Report::new(error).wrap_err(format!(
                "error when trying to get zipball url from github repository `{source_url}`"
            )));
        }
    };

    let zip_filename = match response.header("content-disposition") {
        Some(content_disposition) => {
            tracing::debug!("content-disposition: {content_disposition}");
            match extract_filename(content_disposition) {
                Some(filename) => match filename.strip_suffix(".zip") {
                    Some(filename) => filename.to_string(),
                    None => {
                        eyre::bail!("filename in content-disposition header while collecting zipball for `{source_url}` doesn't end in `.zip`. github might have changed their api.");
                    }
                },

                None => {
                    eyre::bail!("couldn't find filename in content-disposition header while collecting zipball for `{source_url}`. github might have changed their api.");
                }
            }
        }

        None => {
            eyre::bail!("couldn't find content-disposition header while collecting zipball for `{source_url}`. github might have changed their api.");
        }
    };

    let commit = match plugin_lock {
        Some(plugin_lock) => plugin_lock.commit.clone(),

        None => match extract_commit(&zip_filename) {
            Some(commit) => commit,
            None => {
                eyre::bail!("couldn't extract commit from filename `{zip_filename}` while collecting zipball for `{source_url}`. github might have changed their api.");
            }
        },
    };

    if !valid_sha(&commit) {
        eyre::bail!("invalid commit `{commit}` while collecting zipball for `{source_url}`. github might have changed their api.");
    }

    let plugin_dir = downloaded_plugin_dir()?;
    let extract_path = plugin_dir.join(format!("{author}-{repository}-{commit}"));

    if extract_path.exists() {
        if plugin_lock.is_none() {
            lockfile.add(
                source_url,
                PluginLock {
                    commit,
                    sha512: hash_directory(&extract_path)
                        .wrap_err("error hashing pre-existing plugin directory")?,
                },
            );
        }

        return Ok(extract_path);
    }

    let temporary_extract_path = plugin_dir.join(format!("{zip_filename}.tmp"));
    if temporary_extract_path.exists() {
        std::fs::remove_dir_all(&temporary_extract_path).with_context(|| {
            format!(
                "error removing pre-existing temporary directory `{}`",
                temporary_extract_path.display()
            )
        })?;
    }

    std::fs::create_dir_all(&temporary_extract_path).with_context(|| {
        format!(
            "error creating temporary directory `{}`",
            temporary_extract_path.display()
        )
    })?;

    let mut reader = response.into_reader();

    loop {
        let mut file = match zip::read::read_zipfile_from_stream(&mut reader) {
            Ok(Some(file)) => file,
            Ok(None) => break,
            Err(error) => {
                return Err(eyre::Report::new(error).wrap_err(format!(
                    "error downloading zipfile from github repository `{source_url}`",
                )));
            }
        };

        if !file.is_file() {
            continue;
        }

        let enclosed_name = match file.enclosed_name() {
            Some(name) => match strip_directory(name) {
                Some(name) => name,
                None => {
                    eyre::bail!(
                        "zipfile from github repository `{source_url}` contains a file without a parent directory, we were expecting everything to be wrapped in a directory. github might have changed their api."
                    );
                }
            },

            None => {
                eyre::bail!(
                    "zipfile from github repository `{source_url}` contains a file without a name."
                );
            }
        };

        tracing::debug!("extracting file `{}`", enclosed_name.display());

        let path = temporary_extract_path.join(enclosed_name);
        std::fs::create_dir_all(path.parent().unwrap()).with_context(|| {
            format!(
                "error creating directory `{}` while extracting zipfile from github repository `{source_url}`",
                path.parent().unwrap().display(),
            )
        })?;

        let mut out_file = std::fs::File::create(&path).with_context(|| {
            format!(
                "error creating file `{}` while extracting zipfile from github repository `{source_url}`",
                path.display(),
            )
        })?;

        std::io::copy(&mut file, &mut out_file).with_context(|| {
            format!(
                "error copying file `{}` while extracting zipfile from github repository `{source_url}`",
                path.display(),
            )
        })?;
    }

    lockfile.add(
        source_url,
        PluginLock {
            sha512: hash_directory(&temporary_extract_path)
                .wrap_err("error hashing just extracted plugin directory")?,
            commit,
        },
    );

    std::fs::rename(&temporary_extract_path, &extract_path).with_context(|| {
        format!(
            "error renaming temporary directory `{}` to `{}`",
            temporary_extract_path.display(),
            extract_path.display(),
        )
    })?;

    Ok(extract_path)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_commit() {
        assert_eq!(
            super::extract_filename(
                "attachment; filename=Kampfkarren-selene-plugin-hub-test-c49aa82"
            )
            .as_deref()
            .and_then(super::extract_commit),
            Some("c49aa82".to_string())
        );
    }
}
