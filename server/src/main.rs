use battery;
use hostname;
use mpris::
{
    self,
    PlaybackStatus
};
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
    thread,
    time::Duration
};
use sysinfo;
use whoami;

const CONNECTION_ATTEMPTS: u8 = 3;
const DBUS_ADDR_KEY: &str = "DBUS_SESSION_BUS_ADDRESS";
const CONNECTION_SLEEP: Duration = Duration::from_secs(5);

fn flush_stdout() { io::stdout().flush().expect("Couldn't flush stdout"); }

fn detect_dev() -> String
{
    loop
    {
        print!("Trying to detect device... ");
        flush_stdout();
        let mut first_non_arduino = None;
        match serialport::available_ports()
        {
            Ok(ports) =>
            {
                for port in &ports
                {
                    if let SerialPortType::UsbPort(info) = &port.port_type
                    {
                        if info.vid == 0x0403  // Prefer ports with the Arduino vendor ID
                        {
                            let port_name = port.port_name.clone();
                            println!("{} found", port_name);
                            return port_name;
                        }
                        if first_non_arduino == None
                        {
                            first_non_arduino = Some(port.port_name.clone());
                        }
                    }
                }
            }
            Err(why) => eprintln!("Couldn't retrieve ports: {}", why)
        }
        if let Some(first_port_name) = first_non_arduino
        {
            println!("{} found", first_port_name);
            return first_port_name;
        }
        println!("failed");
        thread::sleep(CONNECTION_SLEEP);
    }
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
            println!("Serial connection to {} initialized successfully.\n\
                Transmitting info at 9600 bauds.", dev);
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
    let finite_attempts = env::args().nth(1).is_none();
    let mut attempt = 1;
    loop
    {
        print!("Attempt {}/{}... ",
               attempt,
               if finite_attempts { CONNECTION_ATTEMPTS.to_string() }
               else { String::from("infinite") }
        );
        flush_stdout();
        if let Ok(port) = connect(&dev) { return (dev, port) }
        thread::sleep(CONNECTION_SLEEP);
        if finite_attempts && attempt >= CONNECTION_ATTEMPTS
        {
            dev = detect_dev();
            attempt = 0;
        }
        attempt += 1;
    }
}

fn read_cpu_and_memory(sys: &mut sysinfo::System, components: &mut sysinfo::Components) -> String
{
    sys.refresh_cpu_all();
    sys.refresh_memory();
    components.refresh_list();
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

fn read_battery_and_host(
    battery_manager: &battery::Manager,
    user: &String,
    host: &str,
    times_displayed: &u8
) -> String
{
    let mut batteries = battery_manager.batteries()
        .expect("Couldn't retrieve batteries");
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

fn read_music() -> Option<String>
{
    let player_finder = mpris::PlayerFinder::new()
        .expect("Couldn't retrieve playing music");
    if let Ok(player) = player_finder.find_active()
    {
        let playing_char =
            if let Ok(PlaybackStatus::Playing) = player.get_playback_status() { "#" }
            else { "$" };
        if let Ok(metadata) = player.get_metadata()
        {
            let artists = metadata
                .artists()
                .filter(|artist| !artist.is_empty())
                .unwrap_or(vec!["Unknown artist"]);
            let title = metadata.title().unwrap_or("Unknown title");
            return Some(format!("{} {};{}", playing_char, artists.join(", "), title));
        }
        return Some(format!("{} No music data;", playing_char));
    }
    None
}

fn main()
{
    let mut dev = env::args().nth(1).unwrap_or_else(|| detect_dev());
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
    let battery_manager = battery::Manager::new()
        .expect("Couldn't create instance of battery::Manager");
    let user = whoami::username();
    let host = hostname::get()
        .expect("Couldn't retrieve hostname")
        .to_string_lossy()
        .into_owned();

    if let Ok(_) = env::var(DBUS_ADDR_KEY).map(|addr| addr.contains("unix:abstract"))
    {
        env::set_var(DBUS_ADDR_KEY, "unix:path=/run/user/1000/bus");
    }

    let mut screen = 0;
    let mut times_displayed: u8 = 0;
    loop
    {
        if times_displayed > 5
        {
            screen += 1;
            if screen > 2 { screen = 0; }
            times_displayed = 0;
        }
        let content;
        match screen
        {
            0 => content = read_cpu_and_memory(&mut sys, &mut components),
            1 => content = read_battery_and_host(&battery_manager, &user, &host, &times_displayed),
            2 =>
            {
                match read_music()
                {
                    Some(music_content) => content = music_content,
                    None =>
                    {
                        screen = 0;
                        content = read_cpu_and_memory(&mut sys, &mut components);
                    }
                }
            },
            _ => panic!("No matching screen")
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