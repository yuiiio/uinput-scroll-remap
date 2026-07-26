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

    // 状態保持用変数（ループの外に配置します）
    // 前回のスクロール値（1: 上, -1: 下, 0: 未検出/リセット）
    let mut last_wheel_dir: i32 = 0;

    // 3. イベントループ
    loop {
        for event in phys_mouse.fetch_events()? {
            let swapped_event = event;

            match event.kind() {
                // 相対座標（REL）のX軸とY軸を入れ替える
                InputEventKind::RelAxis(axis) => {
                    match axis {
                        // ★ スクロールホイール（縦スクロール）の制御を追加
                        RelativeAxisType::REL_WHEEL => {
                            let current_dir = event.value(); // 通常、1 (上) か -1 (下)

                            if current_dir == last_wheel_dir && current_dir != 0 {
                                // 前回の方向と同じ（2回連続入力された！）
                                // この時だけイベントをそのまま流し、判定をリセットする
                                last_wheel_dir = 0; 
                            } else {
                                // 1回目の入力、または逆方向への入力
                                // 今回の方向を記憶して、このイベントの出力をスキップする
                                last_wheel_dir = current_dir;
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
