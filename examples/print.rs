use ez_input::RinputerHandle;

fn main() {
    let mut handle = RinputerHandle::open()
        .expect("Failed opening rinputer3 device");

    loop {
        let event = handle.get_event_blocking().unwrap();
        println!("{:#?}", event);
    }
}
