use std::{
    net::UdpSocket,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static NIST: &str = "132.163.97.6";

fn main() {
    // :0 - port gets assigned by the OS
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let request: [u8; 48] = [
        0x23, 0x00, 0x00, 0x00, // LI, VN, Mode, Stratum, Poll, Precision
        0x00, 0x00, 0x00, 0x00, // Root delay
        0x00, 0x00, 0x00, 0x00, // Root dispersion
        0x00, 0x00, 0x00, 0x00, // Reference identifier
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reference timestamp
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Originate timestamp
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Receive timestamp
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Transmit timestamp
    ];

    socket.send_to(&request, format!("{}:123", NIST)).unwrap();
    println!("Packet sent!");

    let mut buf = [0; 48];
    let (amt, src) = socket.recv_from(&mut buf).unwrap();

    let leap_indicator = buf[0] >> 6;
    let version = (buf[0] & 0b00111000) >> 3;
    let mode = buf[0] & 0b00000111;
    let stratum_level = buf[1];
    let polling_interval = buf[2];
    let precision = buf[3];

    let root_delay = ntp_duration(u32::from_be_bytes(buf[4..8].try_into().unwrap()));
    let root_dispersion = ntp_duration(u32::from_be_bytes(buf[8..12].try_into().unwrap()));
    let reference_id = std::str::from_utf8(&buf[12..16]).unwrap();
    let reference_timestamp = ntp_timestamp(u64::from_be_bytes(buf[16..24].try_into().unwrap()));
    let origin_timestamp = ntp_timestamp(u64::from_be_bytes(buf[24..32].try_into().unwrap()));
    let receive_timestamp = ntp_timestamp(u64::from_be_bytes(buf[32..40].try_into().unwrap()));
    let transmit_timestamp = ntp_timestamp(u64::from_be_bytes(buf[40..48].try_into().unwrap()));

    println!("received {} byte from {}:\n {:?}", amt, src, buf);
    println!(
        "leap indicator: {}, version: {}, mode: {}",
        leap_indicator, version, mode
    );
    println!("stratum level: {:x?}", stratum_level);
    println!("polling interval: {}s", 2_i32.pow(polling_interval as u32));
    println!("precision: {}s", 2_f32.powf(precision as i8 as f32));
    println!("root delay: {:?}", root_delay);
    println!("root dispersion: {:?}", root_dispersion);
    println!("reference ID: {:?}", reference_id);
    println!("reference timestamp: {:?}", reference_timestamp);
    println!("receive timestamp: {:?}", receive_timestamp);
    println!("transmit timestamp: {:?}", transmit_timestamp);

    let reference_timestamp = u64::from_be_bytes(buf[16..24].try_into().unwrap());
    let origin_timestamp = u64::from_be_bytes(buf[24..32].try_into().unwrap());
    let receive_timestamp = u64::from_be_bytes(buf[32..40].try_into().unwrap());
    let transmit_timestamp = u64::from_be_bytes(buf[40..48].try_into().unwrap());

    // Calculate the clock offset between the client and server
    let round_trip_delay = (current_ntp_timestamp as i64
        - response_packet.originate_timestamp as i64)
        - (response_packet.transmit_timestamp as i64 - response_packet.receive_timestamp as i64);
    let clock_offset = ((response_packet.receive_timestamp as i64
        - response_packet.originate_timestamp as i64)
        + (response_packet.transmit_timestamp as i64 - current_ntp_timestamp as i64))
        / 2;

    // Calculate the corrected time using the clock offset and round trip delay
    let corrected_ntp_timestamp = current_ntp_timestamp as i64 + clock_offset;
    let corrected_system_time =
        UNIX_EPOCH + Duration::new((corrected_ntp_timestamp - NTP_EPOCH_OFFSET) as u64, 0);

    // Print the corrected system time
    println!("{:?}", corrected_system_time);
}

fn ntp_duration(val: u32) -> Duration {
    let seconds: u64 = (val >> 16) as u64;
    let fraction: u64 = (val & 0xFFFF) as u64;
    let nanos: u64 = (fraction * 1_000_000_000) / 65536;
    Duration::new(seconds, nanos as u32)
}

fn ntp_timestamp(val: u64) -> SystemTime {
    let seconds_since_ntp_epoch = (val >> 32) as u64;
    let seconds_since_unix_epoch = seconds_since_ntp_epoch - 2_208_988_800;
    let fractional_seconds = (val & 0xFFFFFFFF) as u64;
    let nanos = (fractional_seconds * 1_000_000_000) >> 32;
    UNIX_EPOCH + Duration::new(seconds_since_unix_epoch, nanos as u32)
}
