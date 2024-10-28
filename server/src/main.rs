use battery::Manager;
use hostname;
use serial::
{
    prelude::*,
    SystemPort
};
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

fn input_dev() -> String
{
    let default_dev = String::from("/dev/ttyUSB0");
    print!("Enter client device (leave blank for {}): ", default_dev);
    stdout().flush().expect("Couldn't flush stdout");
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Couldn't read USER input");
    let trimmed_input = input.trim();
    if trimmed_input == "" { default_dev } else { String::from(trimmed_input) }
}

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
        dev = input_dev();
    }
    dev
}

fn reconfigure_port(port: &mut SystemPort)
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

fn read_battery_and_network(battery_manager: &Manager, user: &String, host: &str, times_displayed: &u8) -> String
{
    let mut batteries = battery_manager.batteries().expect("Couldn't retrieve batteries");
    let battery = batteries.next().expect("Couldn't retrieve battery").expect("This lib really likes Rust safety with expect()");
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
    let mut dev = get_dev();
    let mut port = serial::open(&dev).expect("Couldn't open serial connection");
    reconfigure_port(&mut port);
    println!("Serial connection to {} initialized successfully.\nTransmitting info at 9600 bauds.", dev);

    let mut sys = System::new();
    let mut components = Components::new();
    let battery_manager = Manager::new().expect("Couldn't create instance of battery::Manager");
    let user = whoami::username();
    let host = hostname::get().expect("Couldn't retrieve hostname").to_string_lossy().into_owned();
    let mut screen = true;
    let mut times_displayed: u8 = 0;
    loop
    {
        if times_displayed > 4        {
            screen = !screen;
            times_displayed = 0;
        }
        let content;
        if screen
        {
            sys.refresh_cpu_all();
            sys.refresh_memory();
            components.refresh_list();
            content = read_cpu_and_memory(&sys, &components);
        }
        else {
            content = read_battery_and_network(&battery_manager, &user, &host, &times_displayed);
        }
        print!("{}      \r", content);
        stdout().flush().expect("Couldn't flush stdout");
        match port.write(format!("{}\n", content).as_bytes())
        {
            Ok(_) => {}
            Err(why) =>
            {
                println!("{}. Attempting to reconnect to {}...", why, dev);
                'reconnector: loop
                {
                    for n in 0..5
                    {
                        stdout().flush().expect("Couldn't flush stdout");
                        match serial::open(&dev)
                        {
                            Ok(new_port) =>
                            {
                                port = new_port;
                                reconfigure_port(&mut port);
                                println!("Attempt {}/5 succeeded: connection reestablished successfully.", n + 1);
                                break 'reconnector;
                            }
                            Err(why) => print!("Attempt {}/5 failed: {}.\r", n + 1, why)
                        }
                        thread::sleep(Duration::from_secs(5));
                    }
                    println!();
                    dev = input_dev();
                }
            }
        }
        times_displayed += 1;
        thread::sleep(Duration::from_secs(1));
    }
}