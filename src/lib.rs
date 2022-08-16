use evdev::{
    Device,
    Error,
    AbsoluteAxisType,
    Key,
    InputEventKind,
};

#[derive(Debug, PartialEq)]
pub enum EzEvent {
    DirectionUp,
    DirectionDown,
    DirectionLeft,
    DirectionRight,

    North(bool),
    South(bool),
    East(bool),
    West(bool),

    Select(bool),
    Start(bool),

    R(bool),
    L(bool),
    R2(i32),
    L2(i32),
}

pub struct RinputerHandle {
    evdev_device: Device,
    pressed_leftright: bool,
    pressed_updown: bool,
}

fn adc(val: i32, deadzone_min: i32, press: &mut bool, less: EzEvent, more: EzEvent) -> Option<EzEvent> {
    if val > deadzone_min || val < -deadzone_min {
        if *press {
            return None;
        } else {
            *press = true;
            if val > 0 {
                return Some(more);
            } else {
                return Some(less);
            }
        }
    } else {
        *press = false;
    }
    None
}

impl RinputerHandle {
    pub fn open() -> Option<RinputerHandle> {
        for (_, candidate) in evdev::enumerate() {
            if candidate.input_id().version() == 0x2137 {
                return Some(RinputerHandle {
                    evdev_device: candidate,
                    pressed_leftright: false,
                    pressed_updown: false,
                });
            }
        }
        None
    }
    pub fn get_event_blocking(&mut self) -> Result<EzEvent, Error> {
        loop {
            for ev in self.evdev_device.fetch_events()? {
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        let val = ev.value() != 0;
                        match key {
                            Key::BTN_NORTH  => return Ok(EzEvent::North(val)),
                            Key::BTN_SOUTH  => return Ok(EzEvent::North(val)),
                            Key::BTN_EAST   => return Ok(EzEvent::East(val)),
                            Key::BTN_WEST   => return Ok(EzEvent::West(val)),
                            Key::BTN_TR     => return Ok(EzEvent::R(val)),
                            Key::BTN_TL     => return Ok(EzEvent::L(val)),
                            Key::BTN_START  => return Ok(EzEvent::Start(val)),
                            Key::BTN_SELECT => return Ok(EzEvent::Select(val)),
                            _               => continue,
                        }
                    },
                    InputEventKind::AbsAxis(abs) => {
                        // we can afford ourselves to not check minmax, as they are
                        // hardcoded in Rinputer3 to be the same
                        let val = ev.value();
                        if let Some(ev) = match abs {
                            AbsoluteAxisType::ABS_X     => adc(val, 20000, &mut self.pressed_leftright, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_Y     => adc(val, 20000, &mut self.pressed_updown, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_HAT0X => adc(val, 0,     &mut self.pressed_leftright, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_HAT0Y => adc(val, 0,     &mut self.pressed_updown, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_Z     => Some(EzEvent::L2(val)),
                            AbsoluteAxisType::ABS_RZ    => Some(EzEvent::R2(val)),
                            _ => continue,
                        } {
                            return Ok(ev);
                        } else {
                            continue;
                        }
                    },
                    _ => continue,
                }
            }
        }
    }
}
