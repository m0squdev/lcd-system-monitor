use battery;
use hostname;
use serial::prelude::*;
use std::
{
    env,
    io::
    {
        stdin,
        stdout,
        Write
    },
    thread,
    time::Duration
};
use sysinfo::
{
    Components,
    System
};
use whoami;

fn get_dev() -> String
{
    let dev;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1
    {
        dev = args[1].clone();
    }
    else
    {
        let dev_prefix = String::from("/dev/tty");
        print!("Enter client device (leave blank for /dev/ttyUSB0\n{}", dev_prefix);
        stdout().flush().expect("Couldn't flush stdout");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Couldn't read user input");
        let mut dev_suffix = input.trim();
        if dev_suffix.is_empty()
        {
            dev_suffix = "USB0";
        }
        dev = dev_prefix + dev_suffix;  // You can use the "+" operator to concatenate a String and a str
    }
    dev
}

fn read_cpu_and_memory(sys: &System, components: &Components) -> String
{
    let cpu_usage = sys.global_cpu_usage();
    let mut temperatures: Vec<f32> = Vec::new();
    for component in components
    {
        if format!("{:?}", component).contains("Core")
        {
            temperatures.push(component.temperature());
        }
    }
    let avg_temperature = temperatures.iter().sum::<f32>() / temperatures.len() as f32;

    let memory_usage = sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0;
    let swap_usage = sys.used_swap() as f32 / sys.total_swap() as f32 * 100.0;

    let line1 = format!("CPU {:.0}% {:.0}^C", cpu_usage, avg_temperature);
    let line2 = format!("RAM {:.0}% Swp {:.0}%", memory_usage, swap_usage);
    format!("{};{}", line1, line2)
}

fn read_battery_and_network() -> String
{
    let battery_manager = battery::Manager::new().expect("Couldn't create instance of battery::Manager");
    let mut batteries = battery_manager.batteries().expect("Couldn't retrieve batteries");
    let battery = batteries.next().expect("Couldn't retrieve battery").expect("This lib really likes Rust safety with expect()");
    let battery_state = battery.state().to_string();
    let battery_state_symbols;
    if battery_state == "charging"
    {
        battery_state_symbols = "` ";
    }
    else {
        battery_state_symbols = "";
    }
    let battery_percentage = battery.state_of_charge().value * 100.0;
    let user = whoami::username();

    let host = hostname::get().expect("Couldn't retrieve hostname").to_string_lossy().into_owned();

    let line1 = format!("{}{:.0}% Usr:{}", battery_state_symbols, battery_percentage, user);
    let line2 = format!("{}", host);
    format!("{};{}", line1, line2)
}

fn main()
{
    let dev = get_dev();
    let mut port = serial::open(&dev).expect("Couldn't open serial connection");
    port.reconfigure(&|settings|
    {
        settings.set_baud_rate(serial::Baud9600).expect("Couldn't set baud rate");
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }).expect("Couldn't configure serial connection");
    println!("Serial connection to {} initialized successfully.\nTransmitting info at 9600 bauds.", dev);

    let mut sys = System::new();
    let mut components = Components::new();
    let mut screen = true;
    let mut times_displayed = 0;
    loop
    {
        if times_displayed > 4        {
            screen = !screen;
            times_displayed = 0;
        }
        let content;
        if screen
        {
            sys.refresh_all();
            components.refresh_list();
            content = read_cpu_and_memory(&sys, &components);
        }
        else {
            content = read_battery_and_network();
        }
        print!("{}      \r", content);
        stdout().flush().expect("Couldn't flush stdout");
        port.write(format!("{}\n", content).as_bytes()).expect("Couldn't write to serial");
        times_displayed += 1;
        thread::sleep(Duration::from_secs(1));
    }
}