use std::fs;
use std::path::Path;

use crate::index::{Index, IndexEntry};
use crate::object::blob::Blob;
use crate::object::store;
use crate::object::tree;
use crate::refs;

pub fn execute(branch: &str) -> Result<(), String> {
    // ブランチの存在確認
    let target_ref = format!("./.minigit/refs/heads/{}", branch);
    if !Path::new(&target_ref).exists() {
        return Err(format!("branch '{}' not found", branch));
    }

    // current tree と target tree のファイル一覧を取得
    let current_entries = match refs::current_tree_hash() {
        Some(hash) => tree::collect_entries(&hash).unwrap_or_default(),
        None => Vec::new(),
    };

    let target_commit =
        fs::read_to_string(&target_ref).map_err(|e| format!("failed to read branch ref: {}", e))?;
    let target_commit = target_commit.trim();
    let (_, commit_body) = store::load(target_commit)?;
    let target_tree_hash = crate::object::commit::Commit::parse_tree_hash(&commit_body)?;
    let target_entries = tree::collect_entries(&target_tree_hash)?;

    // 安全性チェック: target で変更・削除されるファイルに未保存の変更がないか
    check_safety(&current_entries, &target_entries)?;

    // ワーキングディレクトリの更新
    update_working_directory(&current_entries, &target_entries)?;

    // HEAD を書き換え
    fs::write("./.minigit/HEAD", format!("ref: refs/heads/{}\n", branch))
        .map_err(|e| format!("failed to update HEAD: {}", e))?;

    // index を target tree の内容で更新
    update_index(&target_entries)?;

    Ok(())
}

fn check_safety(current: &[(String, String)], target: &[(String, String)]) -> Result<(), String> {
    for (path, current_hash) in current {
        let target_hash = target.iter().find(|(p, _)| p == path).map(|(_, h)| h);

        // target で変更・削除されるファイルか？
        let will_change = match target_hash {
            Some(h) if h == current_hash => false, // 同じ内容 → 変更なし
            _ => true,                             // 違う or 削除される
        };

        if !will_change {
            continue;
        }

        // ワーキングディレクトリの内容が current tree と一致するか？
        match Blob::from_file(path) {
            Ok(blob) => {
                let working_hash = store::hash(&blob);
                if working_hash != *current_hash {
                    return Err(format!(
                        "error: Your local changes to '{}' would be overwritten by checkout.\nPlease commit your changes or stash them before you switch branches.",
                        path
                    ));
                }
            }
            Err(_) => {
                // ファイルが存在しない場合、current tree にはあるので変更とみなす
                // ただし target でも削除されるなら問題ない
                if target_hash.is_some() {
                    return Err(format!(
                        "error: Your local changes to '{}' would be overwritten by checkout.",
                        path
                    ));
                }
            }
        }
    }

    // target にあって current にないファイルが、未追跡ファイルとして存在しないか
    for (path, _) in target {
        if current.iter().any(|(p, _)| p == path) {
            continue; // current にもある → 上のチェックで対応済み
        }
        if Path::new(path).exists() {
            return Err(format!(
                "error: The following untracked file would be overwritten by checkout:\n  {}",
                path
            ));
        }
    }

    Ok(())
}

fn update_working_directory(
    current: &[(String, String)],
    target: &[(String, String)],
) -> Result<(), String> {
    // current にあって target にない → 削除
    for (path, _) in current {
        if !target.iter().any(|(p, _)| p == path) {
            let _ = fs::remove_file(path);
        }
    }

    // target にあるファイルを復元（current と同じハッシュなら何もしない）
    for (path, target_hash) in target {
        let needs_update = match current.iter().find(|(p, _)| p == path) {
            Some((_, current_hash)) if current_hash == target_hash => false,
            _ => true,
        };

        if needs_update {
            restore_file(path, target_hash)?;
        }
    }

    Ok(())
}

fn restore_file(path: &str, hash: &str) -> Result<(), String> {
    // 親ディレクトリを作成
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create directory: {}", e))?;
        }
    }

    let (_, content) = store::load(hash)?;
    fs::write(path, content).map_err(|e| format!("failed to restore file '{}': {}", path, e))
}

fn update_index(target_entries: &[(String, String)]) -> Result<(), String> {
    let mut index = Index {
        entries: Vec::new(),
    };
    for (path, hash) in target_entries {
        index.add(IndexEntry::new(path, hash));
    }
    index.save()
}
