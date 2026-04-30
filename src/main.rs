use anyhow::Result;
use evdev::{Device, InputEventKind, Key, RelativeAxisType};
use std::fs::File;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // 入力デバイス（適宜変更）
    let path = "/dev/input/event2";
    let mut dev = Device::open(path)?;

    println!("Using device: {}", dev.name().unwrap_or("unknown"));

    // grab（他にイベントを流さない）
    dev.grab()?;

    // uinput 仮想デバイス作成
    let mut udev = uinput::default()?
        .name("scroll-remap-virtual-mouse")?
        .event(uinput::event::relative::Position::X)?
        .event(uinput::event::relative::Position::Y)?
        .event(uinput::event::relative::Wheel::Vertical)?
        .event(uinput::event::relative::Wheel::Horizontal)?
        .event(uinput::event::controller::Mouse::Left)?
        .event(uinput::event::controller::Mouse::Right)?
        .event(uinput::event::controller::Mouse::Middle)?
        .create()?;

    let mut scroll_mode = false;

    loop {
        for ev in dev.fetch_events()? {
            match ev.kind() {
                InputEventKind::Key(Key::BTN_MIDDLE) => {
                    scroll_mode = ev.value() != 0;

                    // 中クリック自体も送りたい場合
                    udev.write(
                        uinput::event::controller::Mouse::Middle,
                        ev.value(),
                    )?;
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                    if scroll_mode {
                        // 横スクロール
                        udev.write(
                            uinput::event::relative::Wheel::Horizontal,
                            ev.value(),
                        )?;
                    } else {
                        udev.write(
                            uinput::event::relative::Position::X,
                            ev.value(),
                        )?;
                    }
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_Y) => {
                    if scroll_mode {
                        // 縦スクロール（符号反転すると自然なことが多い）
                        udev.write(
                            uinput::event::relative::Wheel::Vertical,
                            -ev.value(),
                        )?;
                    } else {
                        udev.write(
                            uinput::event::relative::Position::Y,
                            ev.value(),
                        )?;
                    }
                }

                InputEventKind::Key(Key::BTN_LEFT) => {
                    udev.write(
                        uinput::event::controller::Mouse::Left,
                        ev.value(),
                    )?;
                }

                InputEventKind::Key(Key::BTN_RIGHT) => {
                    udev.write(
                        uinput::event::controller::Mouse::Right,
                        ev.value(),
                    )?;
                }

                _ => {}
            }
        }

        udev.synchronize()?;

        // CPU食いすぎ防止
        thread::sleep(Duration::from_millis(1));
    }
}
