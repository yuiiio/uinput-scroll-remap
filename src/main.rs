use anyhow::Result;
use evdev::{Device, InputEventKind, Key, RelativeAxisType, EventType};
use std::thread;
use std::time::Duration;

fn emit(dev: &mut uinput::Device, ty: EventType, code: u16, val: i32) -> anyhow::Result<()> {
    dev.write(ty.0 as i32, code as i32, val)?;
    Ok(())
}

fn main() -> Result<()> {
    let path = "/dev/input/event2";
    let mut dev = Device::open(path)?;

    println!("Using device: {}", dev.name().unwrap_or("unknown"));

    dev.grab()?;

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
                InputEventKind::Key(Key::BTN_SIDE) => {
                    scroll_mode = ev.value() != 0;

                    // emit(&mut udev, EventType::KEY, Key::BTN_SIDE.0, ev.value())?;
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                    if scroll_mode {
                        emit(&mut udev, EventType::RELATIVE, RelativeAxisType::REL_HWHEEL.0, -ev.value()/4)?;
                    } else {
                        emit(&mut udev, EventType::RELATIVE, RelativeAxisType::REL_X.0, ev.value())?;
                    }
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_Y) => {
                    if scroll_mode {
                        emit(&mut udev, EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, ev.value()/4)?;
                    } else {
                        emit(&mut udev, EventType::RELATIVE, RelativeAxisType::REL_Y.0, ev.value())?;
                    }
                }

                // 既知のもの
                InputEventKind::Key(k) => {
                    emit(&mut udev, EventType::KEY, k.0, ev.value())?;
                }
                _ => {}
            }
        }

        udev.synchronize()?;

        // CPU食いすぎ防止
        thread::sleep(Duration::from_millis(1));
    }
}
