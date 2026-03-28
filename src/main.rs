use std::env;
use minigit::init;
use minigit::common;

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

    println!("不明なコマンド: {}", args[1]);

}