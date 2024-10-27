use serial::prelude::*;
use std::
{
    env,
    io::{
        stdin,
        stdout,
        Write
    },
    thread,
    time::Duration
};
use sysinfo::{
    Components,
    System
};

fn main()
{
    let dev;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1
    {
        dev = args[1].clone();
    }
    else {
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
    let mut port = serial::open(&dev).expect("Couldn't open serial connection");
    port.reconfigure(&|settings|
    {
        settings.set_baud_rate(serial::Baud9600).expect("Couldn't set baud rate");
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        Ok(())
    }).expect("Couldn't configure serial connection");
    println!("Serial connection initialized successfully.\nTransmitting info at 9600 bauds.");

    let mut sys = System::new();
    loop
    {
        sys.refresh_all();
        let cpu_usage = sys.global_cpu_usage();
        let components = Components::new_with_refreshed_list();
        let cpu_temp = components[0].temperature();
        let memory_usage = sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0;
        let swap_usage = sys.used_swap() as f32 / sys.total_swap() as f32 * 100.0;
        let line1 = format!("CPU%{:.0} Temp {}", cpu_usage, cpu_temp);
        let line2 = format!("Mem%{:.0} Swp%{:.0}", memory_usage, swap_usage);
        let content = format!("{};{}", line1, line2);
        print!("{}     \r", content);
        stdout().flush().expect("Couldn't flush stdout");
        port.write(format!("{}\n", content).as_bytes()).expect("Couldn't write to serial");
        thread::sleep(Duration::from_secs(1));
    }
}
