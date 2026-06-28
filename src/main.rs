use evdev::{Device, InputEvent, InputEventKind, RelativeAxisType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. 物理マウスのデバイスファイルを開く (環境に合わせて変更してください)
    let mut phys_mouse = Device::open("/dev/input/event4")?;
    phys_mouse.grab()?; // 他のアプリに生の入力がいかないように独占

    // 2. uinputで仮想マウスを作成
    let mut virtual_mouse = evdev::uinput::VirtualDeviceBuilder::new()?
        .name("Swapped Virtual Mouse")
        .with_keys(&phys_mouse.supported_keys().unwrap_or_default())?
        .with_relative_axes(&phys_mouse.supported_relative_axes().unwrap_or_default())?
        .build()?;

    println!("軸入れ替えマウスの監視を開始しました...");

    // 3. イベントループ
    loop {
        for event in phys_mouse.fetch_events()? {
            let mut swapped_event = event;
            // Linuxカーネルのイベントタイプ「EV_REL (相対座標)」を取得
            let ev_type = event.event_type(); 

            match event.kind() {
                // 相対座標（REL）のX軸とY軸を入れ替える
                InputEventKind::RelAxis(axis) => {
                    match axis {
                        RelativeAxisType::REL_X => {
                            // X軸の動きを、Y軸のコード（REL_Yの生値）で新しく作り直す
                            swapped_event = InputEvent::new(
                                ev_type,
                                RelativeAxisType::REL_Y.0,
                                event.value() * -1,
                            );
                        }
                        RelativeAxisType::REL_Y => {
                            // Y軸の動きを、X軸のコード（REL_Xの生値）で新しく作り直す
                            swapped_event = InputEvent::new(
                                ev_type,
                                RelativeAxisType::REL_X.0,
                                event.value(),
                            );
                        }
                        _ => {}
                    }
                },
                // ボタン（キー入力）の入れ替え
                InputEventKind::Key(key) => {
                    match key {
                        evdev::Key::BTN_LEFT => {
                            swapped_event = InputEvent::new(
                                ev_type,
                                evdev::Key::BTN_RIGHT.code(),
                                event.value(),
                            );
                        }
                        evdev::Key::BTN_RIGHT => {
                            swapped_event = InputEvent::new(
                                ev_type,
                                evdev::Key::BTN_MIDDLE.code(),
                                event.value(),
                            );
                        }
                        evdev::Key::BTN_MIDDLE => {
                            swapped_event = InputEvent::new(
                                ev_type,
                                evdev::Key::BTN_LEFT.code(),
                                event.value(),
                            );
                        }
                        evdev::Key::BTN_SIDE => {
                            swapped_event = InputEvent::new(
                                ev_type,
                                evdev::Key::BTN_MIDDLE.code(),
                                event.value(),
                            );
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
