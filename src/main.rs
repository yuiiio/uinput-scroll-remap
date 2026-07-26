use evdev::{Device, InputEvent, InputEventKind, RelativeAxisType};
use std::env;
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. コマンドライン引数を取得
    let args: Vec<String> = env::args().collect();

    // 第一引数があればそれをデバイスパスとし、無ければデフォルト値を使う
    let device_path = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("使い方: sudo cargo run -- /dev/input/eventX");
        process::exit(1);
    };

    println!("デバイスファイルを開いています: {}", device_path);

    // 2. 物理マウスのデバイスファイルを開く
    let mut phys_mouse = match Device::open(device_path) {
        Ok(dev) => dev,
        Err(e) => {
            eprintln!("エラー: デバイス '{}' を開けませんでした: {}", device_path, e);
            eprintln!("使い方: sudo cargo run -- /dev/input/eventX");
            process::exit(1);
        }
    };

    phys_mouse.grab()?; // 他のアプリに生の入力がいかないように独占

    // 2. uinputで仮想マウスを作成
    let mut virtual_mouse = evdev::uinput::VirtualDeviceBuilder::new()?
        .name("Swapped Virtual Mouse")
        .with_keys(&phys_mouse.supported_keys().unwrap_or_default())?
        .with_relative_axes(&phys_mouse.supported_relative_axes().unwrap_or_default())?
        .build()?;

    println!("軸入れ替えマウスの監視を開始しました...");

    // スクロール量の蓄積用変数（ループの外に配置）
    let mut wheel_accumulator: i32 = 0;
    let mut wheel_hi_res_accumulator: i32 = 0;

    // 3. イベントループ
    loop {
        for event in phys_mouse.fetch_events()? {
            let mut swapped_event = event;
            let ev_type = event.event_type();

            match event.kind() {
                // 相対座標（REL）のX軸とY軸を入れ替える
                InputEventKind::RelAxis(axis) => {
                    match axis {
                        // ★ 通常スクロール (REL_WHEEL)
                        RelativeAxisType::REL_WHEEL => {
                            wheel_accumulator += event.value();
                            let output_val = wheel_accumulator / 2; // 移動量を1/2にする

                            if output_val != 0 {
                                wheel_accumulator %= 2; // 出力に使わなかった余りを蓄積に残す
                                swapped_event = InputEvent::new(
                                    ev_type,
                                    RelativeAxisType::REL_WHEEL.0,
                                    output_val,
                                );
                            } else {
                                // まだ1/2に達していない場合は出力をスキップ
                                continue;
                            }
                        }

                        // ★ 高解像度スクロール (REL_WHEEL_HI_RES)
                        RelativeAxisType::REL_WHEEL_HI_RES => {
                            wheel_hi_res_accumulator += event.value();
                            let output_val = wheel_hi_res_accumulator / 2; // 移動量を1/2にする

                            if output_val != 0 {
                                wheel_hi_res_accumulator %= 2; // 出力に使わなかった余りを蓄積に残す
                                swapped_event = InputEvent::new(
                                    ev_type,
                                    RelativeAxisType::REL_WHEEL_HI_RES.0,
                                    output_val,
                                );
                            } else {
                                // まだ1/2に達していない場合は出力をスキップ
                                continue;
                            }
                        }
                        _ => {}
                    }
                },
                _ => {},
            }

            // 4. 仮想マウスへイベントを書き出す
            virtual_mouse.emit(&[swapped_event])?;
        }
    }
}
