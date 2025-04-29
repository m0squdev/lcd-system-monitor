use battery;
use mpris::
{
    self,
    PlaybackStatus
};
use nvml_wrapper::
{
    self,
    enum_wrappers::device::TemperatureSensor,
    Nvml
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

const CONNECTION_ATTEMPTS: u8 = 3;
const MAX_TIMES_DISPLAYED: u8 = 5;
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

fn get_screen0_content(sys: &mut sysinfo::System, components: &mut sysinfo::Components) -> String
{
    sys.refresh_cpu_all();
    components.refresh();
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
    let line1 = format!("CPU {:.0}% {:.0}^C", cpu_usage, avg_temperature);

    sys.refresh_memory();
    let memory_usage = sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0;
    let swap_usage = sys.used_swap() as f32 / sys.total_swap() as f32 * 100.0;
    let line2 = format!("Mem {:.0}% Swp {:.0}%", memory_usage, swap_usage);

    format!("{};{}", line1, line2)
}

fn get_screen1_content(nvml_device: &nvml_wrapper::Device) -> String
{
    let line1 =
    {
        let gpu_usage = nvml_device
            .utilization_rates()
            .expect("Couldn't retrieve GPU usage")
            .gpu;
        let gpu_temperature = nvml_device
            .temperature(TemperatureSensor::Gpu)
            .expect("Couldn't retrieve GPU temperature");
        format!("GPU {}% {}^C", gpu_usage, gpu_temperature)
    };

    let line2 =
    {
        let memory_info = nvml_device
            .memory_info()
            .expect("Couldn't retrieve GPU memory info");
        let memory_usage =
            memory_info.used as f64 / memory_info.total as f64 * 100.0;
        format!("Mem {:.0}%", memory_usage)
    };

    format!("{};{}", line1, line2)
}

fn get_screen2_content(
    battery_manager: &battery::Manager,
    networks: &mut sysinfo::Networks
) -> String
{
    networks.refresh_list();
    let (total_received, total_transmitted) = networks.iter()
        .fold((0, 0), |(received, transmitted), (_, network)|
            (received + network.received(), transmitted + network.transmitted())
        );

    let (line1, line2) =
        if let Some(first_battery) = battery_manager
            .batteries()
            .expect("Couldn't retrieve batteries")
            .next()
        {
            let battery_data = first_battery.expect("Couldn't retrieve battery data");
            let battery_state_symbol =
                if battery_data.state() == battery::State::Charging { "`" } else { "&" };
            let battery_percentage = battery_data.state_of_charge().value * 100.0;
            (format!("{} {:.0}%", battery_state_symbol, battery_percentage),
             format!("] {} [ {} KB/s", total_received / 1000, total_transmitted / 1000))
        }
        else
        {
            (format!("] {} KB/s", total_received / 1000),
             format!("[ {} KB/s", total_transmitted / 1000))
        };
    
    format!("{};{}", line1, line2)
}

fn get_screen3_content() -> Option<String>
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
                .unwrap_or(vec!["Unknown artist"])
                .join(", ");
            let title = metadata.title().unwrap_or("Unknown title");
            return Some(format!(
                "{} {};{}",
                playing_char,
                if artists.is_empty() { String::from("Unknown artist") } else { artists },
                title
            ));
        }
        return Some(format!("{} No music data;", playing_char));
    }
    None
}

fn main()
{
    let mut dev = env::args().nth(1).unwrap_or(detect_dev());
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
    let mut components = sysinfo::Components::new_with_refreshed_list();
    let battery_manager = battery::Manager::new()
        .expect("Couldn't create instance of battery::Manager");
    let mut networks = sysinfo::Networks::new_with_refreshed_list();
    let nvml = Nvml::init();
    let nvml_device = nvml
        .as_ref()
        .ok()
        .and_then(|nvml| nvml.device_by_index(0).ok());

    let mut screen = 0;
    let mut times_displayed = 0;
    loop
    {
        if times_displayed > MAX_TIMES_DISPLAYED
        {
            screen =
                if screen == 0 && nvml_device.is_none() { 2 }
                else if screen >= 3 { 0 }
                else { screen + 1 };
            times_displayed = 0;
        }
        let content;
        match screen
        {
            0 => content = get_screen0_content(&mut sys, &mut components),
            1 =>
            {
                content = get_screen1_content(&nvml_device.as_ref().unwrap());
                // Refresh network info the second before the related screen is displayed
                if times_displayed == MAX_TIMES_DISPLAYED { networks.refresh(); }
            },
            2 => content = get_screen2_content(
                &battery_manager,
                &mut networks
            ),
            3 =>
            {
                if let Some(music_content) = get_screen3_content()
                {
                    content = music_content;
                }
                else
                {
                    screen = 0;
                    content = get_screen0_content(&mut sys, &mut components);
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
