use std::env;

use minigit::commands;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: minigit <コマンド>");
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "init" => commands::init::execute().map(|_| {
            println!("プロジェクトが正常に初期化されました。");
        }),
        "status" => commands::status::execute(),
        "add" => {
            if args.len() < 3 {
                eprintln!("使い方: minigit add <ファイルパス>");
                std::process::exit(1);
            }
            commands::add::execute(&args[2]).map(|_| {
                println!("正常にインデックスに追加されました: {}", &args[2]);
            })
        }
        "commit" => {
            if args.len() < 3 {
                eprintln!("使い方: minigit commit <コミットメッセージ>");
                std::process::exit(1);
            }
            commands::commit::execute(&args[2]).map(|_| {
                println!("正常にコミットされました。");
            })
        }
        "cat-file" => {
            if args.len() < 4 {
                eprintln!("使い方: minigit cat-file <-t|-p> <ハッシュ>");
                std::process::exit(1);
            }
            match args[2].as_str() {
                "-t" => commands::cat_file::show_type(&args[3]).map(|t| println!("{}", t)),
                "-p" => commands::cat_file::show_content(&args[3]).map(|c| print!("{}", c)),
                _ => {
                    eprintln!("不明なオプション: {}", args[2]);
                    std::process::exit(1);
                }
            }
        }
        cmd => {
            eprintln!("不明なコマンド: {}", cmd);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("エラー: {}", e);
        std::process::exit(1);
    }
}
