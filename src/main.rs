use minigit::add;
use minigit::commit;
use minigit::common;
use minigit::init;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: minigit <コマンド>");
        std::process::exit(1);
    }

    if args[1] == "init" {
        println!("プロジェクトを初期化しています...");
        match init::init_create::create_init_file() {
            Ok(_) => {
                println!("プロジェクトが正常に初期化されました。");
            }
            Err(e) => {
                eprintln!("エラー: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if args[1] == "status" {
        match common::index_readed::read_index() {
            Ok(indexs) => {
                for index in indexs {
                    println!("{} {}", index.status, index.path);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
        return;
    }

    if args[1] == "add" {
        if args.len() < 3 {
            eprintln!("使い方: minigit add <ファイルパス>");
            std::process::exit(1);
        }
        let path = &args[2];
        match add::add_execute::add_file(path) {
            Ok(hash_hex) => match add::add_execute::add_index(path, &hash_hex) {
                Ok(_) => {
                    println!("正常にインデックスに追加されました: {}", path);
                }
                Err(e) => {
                    eprintln!("インデックスの追加に失敗しました: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("ファイルの追加に失敗しました: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if args[1] == "commit" {
        if args.len() < 3 {
            eprintln!("使い方: minigit commit <コミットメッセージ>");
            std::process::exit(1);
        }
        let message = &args[2];
        match commit::commit_execute::commit(message) {
            Ok(_) => {
                println!("正常にコミットされました。");
            }
            Err(e) => {
                eprintln!("コミットに失敗しました: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if args[1] == "cat-file" {
        if args.len() < 4 {
            eprintln!("使い方: minigit cat-file <オプション> <ハッシュ>");
            std::process::exit(1);
        }
        let option = &args[2];
        let hash = &args[3];
        match option.as_str() {
            "-t" => match common::helper::cat_file_type(hash) {
                Ok(object_type) => {
                    println!("{}", object_type);
                }
                Err(e) => {
                    eprintln!("エラー: {}", e);
                    std::process::exit(1);
                }
            },
            "-p" => match common::helper::cat_file_print(hash) {
                Ok(content) => {
                    print!("{}", content);
                }
                Err(e) => {
                    eprintln!("エラー: {}", e);
                    std::process::exit(1);
                }
            },
            _ => {
                eprintln!("不明なオプション: {}", option);
                std::process::exit(1);
            }
        }
        return;
    }

    println!("不明なコマンド: {}", args[1]);
}
