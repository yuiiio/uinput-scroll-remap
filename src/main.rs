use anyhow::Result;
use evdev::{Device, InputEventKind, Key, RelativeAxisType, EventType};

fn emit(dev: &mut uinput::Device, ty: EventType, code: u16, val: i32) -> anyhow::Result<()> {
    dev.write(ty.0 as i32, code as i32, val)?;
    Ok(())
}

struct ScrollState {
    acc_x: f32,
    acc_y: f32,
}

impl ScrollState {
    fn new() -> Self {
        Self { acc_x: 0.0, acc_y: 0.0 }
    }

    fn feed(
        &mut self,
        dx: i32,
        dy: i32,
        udev: &mut uinput::Device,
    ) -> anyhow::Result<()> {
        let sensitivity = 0.03; // ←超重要（調整ポイント）

        self.acc_x += dx as f32 * sensitivity;
        self.acc_y += dy as f32 * sensitivity;

        // 横スクロール
        while self.acc_x.abs() >= 1.0 {
            let step = self.acc_x.signum() as i32;

            udev.write(
                EventType::RELATIVE.0 as i32,
                RelativeAxisType::REL_HWHEEL.0 as i32,
                -step,
            )?;

            self.acc_x -= step as f32;
        }

        // 縦スクロール
        while self.acc_y.abs() >= 1.0 {
            let step = self.acc_y.signum() as i32;

            udev.write(
                EventType::RELATIVE.0 as i32,
                RelativeAxisType::REL_WHEEL.0 as i32,
                step,
            )?;

            self.acc_y -= step as f32;
        }

        Ok(())
    }
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
    let mut scroll = ScrollState::new();

    loop {
        for ev in dev.fetch_events()? {
            match ev.kind() {
                InputEventKind::Key(Key::BTN_SIDE) => {
                    scroll_mode = ev.value() != 0;

                    // emit(&mut udev, EventType::KEY, Key::BTN_SIDE.0, ev.value())?;
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                    if scroll_mode {
                        scroll.feed(ev.value(), 0, &mut udev)?;
                    } else {
                        emit(&mut udev, EventType::RELATIVE, RelativeAxisType::REL_X.0, ev.value())?;
                    }
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_Y) => {
                    if scroll_mode {
                        scroll.feed(0, ev.value(), &mut udev)?;
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
    }
}
