use evdev::{
    Device,
    AbsoluteAxisType,
    Key,
    InputEventKind,
    InputEvent
};
use std::sync::mpsc::{
    self,
    TryRecvError,
    Sender,
    Receiver,
};
use std::thread;
use std::time::Duration;

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



pub struct RinputerHandle {
    evdev_device: Device,
    lr: bool,
    ud: bool,
}

impl RinputerHandle {
    pub fn open() -> Option<RinputerHandle> {
        for candidate in evdev::enumerate() {
            if candidate.input_id().version() == 0x2137 {
                return Some(RinputerHandle {
                    evdev_device: candidate,
                    lr: false,
                    ud: false,
                });
            }
        }
        None
    }
    pub fn get_event_blocking(&mut self) -> Option<EzEvent> {
        loop {
            for ev in self.evdev_device.fetch_events().unwrap() {
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        let val = ev.value() != 0;
                        match key {
                            Key::BTN_NORTH  => return Some(EzEvent::North(val)),
                            Key::BTN_SOUTH  => return Some(EzEvent::South(val)),
                            Key::BTN_EAST   => return Some(EzEvent::East(val)),
                            Key::BTN_WEST   => return Some(EzEvent::West(val)),
                            Key::BTN_TR     => return Some(EzEvent::R(val)),
                            Key::BTN_TL     => return Some(EzEvent::L(val)),
                            Key::BTN_START  => return Some(EzEvent::Start(val)),
                            Key::BTN_SELECT => return Some(EzEvent::Select(val)),
                            _ => continue,
                        }
                    },
                    InputEventKind::AbsAxis(abs) => {
                        // we can afford ourselves to not check minmax, as they are
                        // hardcoded in Rinputer3 to be the same
                        let val = ev.value();
                        match abs {
                            AbsoluteAxisType::ABS_X     => return adc(val, 20000, &mut self.lr, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_Y     => return adc(val, 20000, &mut self.ud, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_HAT0X => return adc(val, 0,     &mut self.lr, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_HAT0Y => return adc(val, 0,     &mut self.ud, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_Z     => return Some(EzEvent::L2(val)),
                            AbsoluteAxisType::ABS_RZ    => return Some(EzEvent::R2(val)),
                            _ => continue,
                        };
                    },
                    _ => continue,
                }
            }
        }
    }
}

#[inline]
fn has_key(dev: &Device, key: evdev::Key) -> bool {
    dev.supported_keys().map_or(false, |keys| keys.contains(key))
}

fn handle_input(mut dev: Device, tx: Sender<InputEvent>) {
    loop {
        for ev in dev.fetch_events().unwrap() {
            let ret = tx.send(ev);
        }
    }
}

pub struct AnyHandle {
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    lr: bool,
    ud: bool,
}

impl AnyHandle {
    pub fn open() -> AnyHandle {
        let (tx, rx) = mpsc::channel();

        let closure_tx = tx.clone();

        thread::spawn(move || {
            loop {
                for mut candidate in evdev::enumerate() {
                    if has_key(&candidate, Key::BTN_SOUTH) {
                        let ret = candidate.grab();
                        if ret.is_err() {
                            continue;
                        }

                        let new_tx = closure_tx.clone();
                        thread::spawn(move || handle_input(candidate, new_tx));
                    }
                }
                thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            rx,
            tx,
            lr: false,
            ud: false,
        }
    }

    pub fn get_event_blocking(&mut self) -> Option<EzEvent> {
        loop {
            for ev in self.rx.recv() {
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        let val = ev.value() != 0;
                        match key {
                            Key::BTN_NORTH  => return Some(EzEvent::North(val)),
                            Key::BTN_SOUTH  => return Some(EzEvent::South(val)),
                            Key::BTN_EAST   => return Some(EzEvent::East(val)),
                            Key::BTN_WEST   => return Some(EzEvent::West(val)),
                            Key::BTN_TR     => return Some(EzEvent::R(val)),
                            Key::BTN_TL     => return Some(EzEvent::L(val)),
                            Key::BTN_START  => return Some(EzEvent::Start(val)),
                            Key::BTN_SELECT => return Some(EzEvent::Select(val)),
                            _ => continue,
                        }
                    },
                    InputEventKind::AbsAxis(abs) => {
                        // we can afford ourselves to not check minmax, as they are
                        // hardcoded in Rinputer3 to be the same
                        let val = ev.value();
                        match abs {
                            AbsoluteAxisType::ABS_X     => return adc(val, 20000, &mut self.lr, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_Y     => return adc(val, 20000, &mut self.ud, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_HAT0X => return adc(val, 0,     &mut self.lr, EzEvent::DirectionLeft, EzEvent::DirectionRight),
                            AbsoluteAxisType::ABS_HAT0Y => return adc(val, 0,     &mut self.ud, EzEvent::DirectionUp, EzEvent::DirectionDown),
                            AbsoluteAxisType::ABS_Z     => return Some(EzEvent::L2(val)),
                            AbsoluteAxisType::ABS_RZ    => return Some(EzEvent::R2(val)),
                            _ => continue,
                        };
                    },
                    _ => continue,
                }
            }
        }
    }
}
