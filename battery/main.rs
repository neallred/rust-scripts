use battery;

fn main() -> battery::Result<()> {
    let manager = battery::Manager::new()?;
    if let Some(Ok(battery)) = manager.batteries()?.next() {
        print!("{}", &format!("{:?}", battery.state_of_charge())[..4]);
    };

    Ok(())
}
