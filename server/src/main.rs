use battery;
use hostname;
use serial::
{
    prelude::*,
    SystemPort
};
use serialport::
{
    self,
    SerialPortType
};
use std::
{
    env,
    io::
    {
        self,
        Write
    },
    process::exit,
    thread,
    time::Duration
};
use sysinfo;
use whoami;

const CONNECTION_ATTEMPTS: u8 = 3;

fn flush_stdout()
{
    io::stdout().flush().expect("Couldn't flush stdout");
}

fn detect_dev() -> Option<String>
{
    print!("Trying to detect device... ");
    flush_stdout();
    match serialport::available_ports()
    {
        Ok(ports) =>
        {
            for port in &ports
            {
                if let SerialPortType::UsbPort(info) = &port.port_type
                {
                    if info.vid == 0x0403 {
                        let port_name = port.port_name.clone();
                        println!("{} found", port_name);
                        return Some(port_name);
                    }
                }
            }
        }
        Err(why) => eprintln!("Couldn't retrieve ports: {}", why)
    }
    println!("failed");
    None
}

fn input_dev() -> String
{
    print!("Enter client device: ");
    flush_stdout();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Couldn't read user input");
    let dev = input.trim();
    if dev.is_empty() { exit(0) }
    String::from(dev)
}

fn connect(dev: &String) -> Result<SystemPort, serial::Error>
{
    match serial::open(&dev)
    {
        Ok(mut port) =>
        {
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
            Ok(port)
        }
        Err(why) =>
        {
            println!("Connection failed: {}.", why);
            Err(why)
        }
    }
}

fn auto_reconnect(mut dev: String) -> (String, SystemPort)
{
    loop
    {
        for attempt in 1..=CONNECTION_ATTEMPTS
        {
            print!("Attempt {}/{}... ", attempt, CONNECTION_ATTEMPTS);
            flush_stdout();
            if let Ok(port) = connect(&dev) { return (dev, port); }
            thread::sleep(Duration::from_secs(5));
        }
        dev = detect_dev().unwrap_or_else(|| input_dev());
    }
}

fn read_cpu_and_memory(sys: &sysinfo::System, components: &sysinfo::Components) -> String
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

fn read_battery_and_network(battery_manager: &battery::Manager, user: &String, host: &str, times_displayed: &u8) -> String
{
    let mut batteries = battery_manager.batteries().expect("Couldn't retrieve batteries");
    let battery = batteries
        .next()
        .expect("Couldn't retrieve battery")
        .expect("This lib really likes Rust safety with expect()");
    let battery_state = battery.state().to_string();
    let battery_state_symbol = if battery_state == "charging" { "`" } else { "&" };
    let battery_percentage = battery.state_of_charge().value * 100.0;

    let line1 =
        if times_displayed % 2 == 0 && battery_percentage < 10.0
        {
            format!("{} RECHARGE NOW", battery_state_symbol)
        }
        else
        {
            format!("{} {:.0}% Usr:{}", battery_state_symbol, battery_percentage, user)
        };
    let line2 = format!("{}", host);
    format!("{};{}", line1, line2)
}

fn main()
{
    let mut dev = detect_dev().unwrap_or_else(|| input_dev());
    let mut port;
    match connect(&dev)
    {
        Ok(new_port) => port = new_port,
        Err(why) =>
        {
            println!("{}. Couldn't establish connection to {}.", why, dev);
            (dev, port) = auto_reconnect(dev);
        }
    }

    let mut sys = sysinfo::System::new();
    let mut components = sysinfo::Components::new();
    let battery_manager = battery::Manager::new().expect("Couldn't create instance of battery::Manager");
    let user = whoami::username();
    let host = hostname::get()
        .expect("Couldn't retrieve hostname")
        .to_string_lossy()
        .into_owned();
    let mut sys_monitor_screen = true;
    let mut times_displayed: u8 = 0;
    loop
    {
        if times_displayed > 4
        {
            sys_monitor_screen = !sys_monitor_screen;
            times_displayed = 0;
        }
        let content;
        if sys_monitor_screen
        {
            sys.refresh_cpu_all();
            sys.refresh_memory();
            components.refresh_list();
            content = read_cpu_and_memory(&sys, &components);
        }
        else
        {
            content = read_battery_and_network(&battery_manager, &user, &host, &times_displayed);
        }
        print!("\x1b[2K{}\r", content);
        flush_stdout();
        if let Err(why) = port.write(format!("{}\n", content).as_bytes())
        {
            println!("{}. Attempting to reconnect to {}.", why, dev);
            (dev, port) = auto_reconnect(dev);
        }
        times_displayed += 1;
        thread::sleep(Duration::from_secs(1));
    }
}